//! Getting a finished case in front of people: publish the images to an
//! evidence branch (so a GitHub comment can embed them with no server), upload
//! the bundle to a case-file server, and post or update the issue comment.

use anyhow::{bail, Context, Result};
use base64::Engine;
use clap::Args;
use reenact_core::comment::{render_comment, Links};
use reenact_core::github;
use reenact_core::pipeline;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args, Clone, Default)]
pub struct ShipArgs {
    /// Post (or update) the result as a comment on the issue.
    #[arg(long)]
    pub post: bool,
    /// Push strip/compare images to this branch of the issue's repo and link them from the comment.
    #[arg(long)]
    pub publish_branch: Option<String>,
    /// Case-file server to upload the bundle to.
    #[arg(long, env = "REENACT_SERVER_URL")]
    pub upload_url: Option<String>,
    #[arg(long, env = "REENACT_UPLOAD_TOKEN", hide_env_values = true)]
    pub upload_token: Option<String>,
}

fn token() -> Result<String> {
    for k in ["GH_TOKEN", "GITHUB_TOKEN"] {
        if let Ok(t) = std::env::var(k) {
            if !t.is_empty() {
                return Ok(t);
            }
        }
    }
    let out = Command::new("gh").args(["auth", "token"]).output().context("gh auth token")?;
    anyhow::ensure!(out.status.success(), "no GitHub token available (set GH_TOKEN)");
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Run git with the token supplied through the environment, not argv.
fn git(dir: &Path, token: &str, args: &[&str]) -> Result<String> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(format!("x-access-token:{token}"));
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "http.extraheader")
        .env("GIT_CONFIG_VALUE_0", format!("AUTHORIZATION: basic {b64}"))
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .context("git")?;
    if !out.status.success() {
        bail!("git {} failed: {}", args.first().unwrap_or(&""), String::from_utf8_lossy(&out.stderr).replace(&b64, "***"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn publishable(case_dir: &Path) -> Vec<(PathBuf, String)> {
    let mut v = Vec::new();
    if let Ok(rd) = std::fs::read_dir(case_dir) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n == "case.json" || n.ends_with(".png") {
                v.push((e.path(), n));
            }
        }
    }
    if let Ok(rd) = std::fs::read_dir(case_dir.join("frames")) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n.ends_with(".jpg") || n.ends_with(".png") {
                v.push((e.path(), format!("frames/{n}")));
            }
        }
    }
    v
}

/// Returns the raw.githubusercontent base URL for this case's folder.
pub fn publish_branch(case_dir: &Path, case_id: &str, repo: &str, branch: &str) -> Result<String> {
    let token = token()?;
    let tmp = tempfile::tempdir()?;
    let url = format!("https://github.com/{repo}.git");
    let cloned = git(tmp.path(), &token, &["clone", "--quiet", "--depth", "1", "--branch", branch, &url, "."]).is_ok();
    if !cloned {
        git(tmp.path(), &token, &["init", "--quiet", "-b", branch])?;
        git(tmp.path(), &token, &["remote", "add", "origin", &url])?;
    }
    let dest = tmp.path().join(case_id);
    let _ = std::fs::remove_dir_all(&dest);
    for (src, rel) in publishable(case_dir) {
        let to = dest.join(&rel);
        std::fs::create_dir_all(to.parent().unwrap())?;
        std::fs::copy(&src, &to)?;
    }
    git(tmp.path(), &token, &["add", "-A"])?;
    if git(tmp.path(), &token, &["status", "--porcelain"])?.is_empty() {
        return Ok(format!("https://raw.githubusercontent.com/{repo}/{branch}/{case_id}"));
    }
    let id = ["-c", "user.name=Reenact", "-c", "user.email=reenact@users.noreply.github.com"];
    let msg = format!("evidence for {case_id}");
    let commit = [id.as_slice(), &["commit", "--quiet", "-m", msg.as_str()]].concat();
    git(tmp.path(), &token, &commit)?;
    let mut pushed = git(tmp.path(), &token, &["push", "--quiet", "origin", &format!("HEAD:{branch}")]).is_ok();
    if !pushed {
        // Another job pushed first: rebase once and retry.
        let _ = git(tmp.path(), &token, &["pull", "--quiet", "--rebase", "origin", branch]);
        pushed = git(tmp.path(), &token, &["push", "--quiet", "origin", &format!("HEAD:{branch}")]).is_ok();
    }
    anyhow::ensure!(pushed, "could not push the evidence branch");
    Ok(format!("https://raw.githubusercontent.com/{repo}/{branch}/{case_id}"))
}

pub async fn upload_bundle(case_dir: &Path, case_id: &str, server: &str, token: &str) -> Result<()> {
    let mut buf = Vec::new();
    {
        let mut w = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        for (src, rel) in publishable(case_dir) {
            w.start_file(rel, zip::write::SimpleFileOptions::default())?;
            w.write_all(&std::fs::read(src)?)?;
        }
        w.finish()?;
    }
    let url = format!("{}/api/cases/{}/bundle", server.trim_end_matches('/'), case_id);
    let r = reqwest::Client::new().post(url).bearer_auth(token).body(buf).send().await?;
    anyhow::ensure!(r.status().is_success(), "upload failed: {}", r.status());
    Ok(())
}

/// Publish, upload and comment for a case directory.
pub async fn ship(case_dir: &Path, a: &ShipArgs) -> Result<()> {
    let case = pipeline::load_case(case_dir)?;
    let mut asset_base: Option<String> = None;
    if let (Some(branch), Some(repo)) = (&a.publish_branch, &case.report.repo) {
        match publish_branch(case_dir, &case.id, repo, branch) {
            Ok(b) => asset_base = Some(b),
            Err(e) => eprintln!("[ship] could not publish images: {e:#}"),
        }
    }
    let mut case_url: Option<String> = None;
    if let (Some(server), Some(tok)) = (&a.upload_url, &a.upload_token) {
        match upload_bundle(case_dir, &case.id, server, tok).await {
            Ok(()) => case_url = Some(format!("{}/c/{}", server.trim_end_matches('/'), case.id)),
            Err(e) => eprintln!("[ship] could not upload the bundle: {e:#}"),
        }
    }
    if a.post {
        let (Some(repo), Some(n)) = (case.report.repo.clone(), case.report.number) else {
            bail!("--post needs a case created with --issue");
        };
        let img = |name: &str| asset_base.as_ref().filter(|_| case_dir.join(name).exists()).map(|b| format!("{b}/{name}"));
        let strip_name = case
            .runs
            .iter()
            .rev()
            .find(|r| r.kind == reenact_core::model::RunKind::Reenact)
            .and_then(|r| r.run.strip_file.clone())
            .unwrap_or_else(|| "strip.png".into());
        let strip = img(&strip_name);
        let compare = img("compare.png");
        let body = render_comment(&case, &Links { strip_url: strip.as_deref(), compare_url: compare.as_deref(), case_url: case_url.as_deref() });
        let url = github::upsert_comment(&repo, n, &case.id, &body)?;
        eprintln!("[ship] comment: {url}");
    }
    Ok(())
}
