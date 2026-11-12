//! GitHub access through the `gh` CLI. In Actions `gh` is preinstalled and reads
//! `GH_TOKEN`; locally it uses the logged-in account. One less HTTP client to
//! get wrong, and the token never touches our code.

use crate::comment::MARKER_PREFIX;
use crate::model::Report;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn parse_issue_url(url: &str) -> Result<(String, String, u64)> {
    let u = url::Url::parse(url).context("not a URL")?;
    anyhow::ensure!(u.host_str() == Some("github.com"), "only github.com issue URLs are supported");
    let seg: Vec<&str> = u.path_segments().map(|s| s.collect()).unwrap_or_default();
    anyhow::ensure!(seg.len() >= 4 && seg[2] == "issues", "expected https://github.com/<owner>/<repo>/issues/<n>");
    Ok((seg[0].to_string(), seg[1].to_string(), seg[3].parse().context("issue number")?))
}

fn gh(args: &[&str], stdin: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("gh");
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if stdin.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }
    let mut child = cmd.spawn().context("running gh (is it installed?)")?;
    if let (Some(s), Some(mut si)) = (stdin, child.stdin.take()) {
        si.write_all(s.as_bytes())?;
    }
    let out = child.wait_with_output()?;
    if !out.status.success() {
        bail!("gh {} failed: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn fetch_issue(url: &str) -> Result<Report> {
    let (o, r, n) = parse_issue_url(url)?;
    let raw = gh(&["api", &format!("repos/{o}/{r}/issues/{n}")], None)?;
    let v: Value = serde_json::from_str(&raw)?;
    Ok(Report {
        title: v["title"].as_str().unwrap_or_default().to_string(),
        body: v["body"].as_str().unwrap_or_default().to_string(),
        source_url: Some(url.to_string()),
        repo: Some(format!("{o}/{r}")),
        number: Some(n),
    })
}

/// Create or update the single Reenact comment for `case_id` on an issue.
pub fn upsert_comment(repo: &str, number: u64, case_id: &str, body: &str) -> Result<String> {
    let raw = gh(&["api", &format!("repos/{repo}/issues/{number}/comments"), "--paginate"], None)?;
    // --paginate concatenates JSON arrays: parse leniently.
    let marker = format!("{MARKER_PREFIX}{case_id} -->");
    let mut existing: Option<u64> = None;
    let de = serde_json::Deserializer::from_str(&raw).into_iter::<Value>();
    for page in de.flatten() {
        if let Some(arr) = page.as_array() {
            for c in arr {
                if c["body"].as_str().map_or(false, |b| b.contains(&marker)) {
                    existing = c["id"].as_u64();
                }
            }
        }
    }
    let payload = json!({ "body": body }).to_string();
    let resp = match existing {
        Some(id) => gh(&["api", "-X", "PATCH", &format!("repos/{repo}/issues/comments/{id}"), "--input", "-"], Some(&payload))?,
        None => gh(&["api", "-X", "POST", &format!("repos/{repo}/issues/{number}/comments"), "--input", "-"], Some(&payload))?,
    };
    let v: Value = serde_json::from_str(&resp)?;
    Ok(v["html_url"].as_str().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_urls() {
        let (o, r, n) = parse_issue_url("https://github.com/usememos/memos/issues/5319").unwrap();
        assert_eq!((o.as_str(), r.as_str(), n), ("usememos", "memos", 5319));
        assert!(parse_issue_url("https://evil.com/a/b/issues/1").is_err());
        assert!(parse_issue_url("https://github.com/a/b/pull/1").is_err());
    }
}
