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
    anyhow::ensure!(
        u.host_str() == Some("github.com"),
        "only github.com issue URLs are supported"
    );
    let seg: Vec<&str> = u.path_segments().map(|s| s.collect()).unwrap_or_default();
    anyhow::ensure!(
        seg.len() >= 4 && seg[2] == "issues",
        "expected https://github.com/<owner>/<repo>/issues/<n>"
    );
    Ok((
        seg[0].to_string(),
        seg[1].to_string(),
        seg[3].parse().context("issue number")?,
    ))
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
        bail!(
            "gh {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
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

/// The issue plus the human replies that add detail (the reporter's, and people with write
/// access). Used for the "Needs info" loop: when the reporter answers, Reenact reads the
/// answer together with the original report. Bot comments (including ours) are skipped.
pub fn fetch_issue_with_thread(url: &str) -> Result<Report> {
    let mut report = fetch_issue(url)?;
    let (o, r, n) = parse_issue_url(url)?;
    let issue: Value =
        serde_json::from_str(&gh(&["api", &format!("repos/{o}/{r}/issues/{n}")], None)?)?;
    let author = issue["user"]["login"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let raw = gh(
        &[
            "api",
            &format!("repos/{o}/{r}/issues/{n}/comments"),
            "--paginate",
        ],
        None,
    )?;
    let mut extra = String::new();
    let mut count = 0;
    for page in serde_json::Deserializer::from_str(&raw)
        .into_iter::<Value>()
        .flatten()
    {
        for c in page.as_array().into_iter().flatten() {
            let body = c["body"].as_str().unwrap_or_default();
            let who = c["user"]["login"].as_str().unwrap_or_default();
            let bot = c["user"]["type"].as_str() == Some("Bot") || body.contains(MARKER_PREFIX);
            let trusted = matches!(
                c["author_association"].as_str(),
                Some("OWNER" | "MEMBER" | "COLLABORATOR")
            );
            if bot || body.trim_start().starts_with("/reenact") || !(who == author || trusted) {
                continue;
            }
            if count >= 5 {
                break;
            }
            count += 1;
            let clipped: String = body.chars().take(800).collect();
            extra.push_str(&format!(
                "\n[{}] {}\n",
                if who == author {
                    "reporter"
                } else {
                    "maintainer"
                },
                clipped
            ));
        }
    }
    if !extra.is_empty() {
        report
            .body
            .push_str("\n\n---\nAdditional detail from the issue thread:\n");
        report.body.push_str(&extra);
    }
    Ok(report)
}

pub const LABELS: [(&str, &str, &str); 4] = [
    (
        "reenact:reproduced",
        "D7301F",
        "Reenact reproduced this in a real browser",
    ),
    (
        "reenact:not-reproduced",
        "1F7A4D",
        "Reenact could not reproduce this on the tested version",
    ),
    (
        "reenact:needs-info",
        "B7791F",
        "Reenact needs more detail to reproduce this",
    ),
    (
        "reenact:inconclusive",
        "5B5851",
        "Reenact could not complete a browser run",
    ),
];

/// Keep exactly one `reenact:*` verdict label on the issue. Best effort: labels are a courtesy.
pub fn set_verdict_label(repo: &str, number: u64, verdict_label: &str) {
    for (name, color, desc) in LABELS {
        let _ = gh(
            &[
                "label",
                "create",
                name,
                "--repo",
                repo,
                "--color",
                color,
                "--description",
                desc,
                "--force",
            ],
            None,
        );
    }
    let remove: Vec<&str> = LABELS
        .iter()
        .map(|l| l.0)
        .filter(|l| *l != verdict_label)
        .collect();
    let n = number.to_string();
    let mut args = vec![
        "issue",
        "edit",
        n.as_str(),
        "--repo",
        repo,
        "--add-label",
        verdict_label,
    ];
    for r in &remove {
        args.push("--remove-label");
        args.push(r);
    }
    if gh(&args, None).is_err() {
        // Removing a label the issue does not have can fail on some versions: add only.
        let _ = gh(
            &[
                "issue",
                "edit",
                n.as_str(),
                "--repo",
                repo,
                "--add-label",
                verdict_label,
            ],
            None,
        );
    }
}

/// Create or update the single Reenact comment for `case_id` on an issue.
pub fn upsert_comment(repo: &str, number: u64, case_id: &str, body: &str) -> Result<String> {
    let raw = gh(
        &[
            "api",
            &format!("repos/{repo}/issues/{number}/comments"),
            "--paginate",
        ],
        None,
    )?;
    // --paginate concatenates JSON arrays: parse leniently.
    let marker = format!("{MARKER_PREFIX}{case_id} -->");
    let mut existing: Option<u64> = None;
    let de = serde_json::Deserializer::from_str(&raw).into_iter::<Value>();
    for page in de.flatten() {
        if let Some(arr) = page.as_array() {
            for c in arr {
                if c["body"].as_str().is_some_and(|b| b.contains(&marker)) {
                    existing = c["id"].as_u64();
                }
            }
        }
    }
    let payload = json!({ "body": body }).to_string();
    let resp = match existing {
        Some(id) => gh(
            &[
                "api",
                "-X",
                "PATCH",
                &format!("repos/{repo}/issues/comments/{id}"),
                "--input",
                "-",
            ],
            Some(&payload),
        )?,
        None => gh(
            &[
                "api",
                "-X",
                "POST",
                &format!("repos/{repo}/issues/{number}/comments"),
                "--input",
                "-",
            ],
            Some(&payload),
        )?,
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
