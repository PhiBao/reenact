//! `reenact suite`: re-run every saved reproduction against the live app.
//! A test that used to be fixed and now fails with a confirmed product bug is a
//! regression, and the suite exits non-zero.

use anyhow::Result;
use clap::Args;
use reenact_core::kane::{self, RunOpts};
use reenact_core::model::VerdictKind;
use reenact_core::pipeline::Ctx;
use reenact_core::verdict::judge;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Args)]
pub struct SuiteArgs {
    #[arg(long, default_value = ".reenact.yml")]
    config: PathBuf,
    #[arg(long, default_value = "reenact/tests")]
    dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SuiteRow {
    pub name: String,
    pub kind: VerdictKind,
    pub detail: String,
}

impl SuiteRow {
    pub fn label(&self) -> &'static str {
        match self.kind {
            VerdictKind::NotReproduced => "holds",
            VerdictKind::Reproduced => "REGRESSED",
            VerdictKind::NeedsInfo => "unsure",
            VerdictKind::Inconclusive => "inconclusive",
        }
    }
}

/// Run every `*_test.md` in `dir` (except `exclude`) live, one after another.
pub async fn run_dir(ctx: &Ctx, dir: &Path, exclude: Option<&str>) -> Result<Vec<SuiteRow>> {
    let dir = std::fs::canonicalize(dir)?;
    let mut tests: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("_test.md") && Some(n) != exclude)
        })
        .collect();
    tests.sort();

    let creds = match &ctx.cfg.login {
        Some(l) => vec![
            (
                "username".to_string(),
                std::env::var(&l.username_env)?,
                false,
            ),
            (
                "password".to_string(),
                std::env::var(&l.password_env)?,
                true,
            ),
        ],
        None => vec![],
    };

    let mut rows = Vec::new();
    for t in &tests {
        let name = t.file_name().unwrap().to_string_lossy().to_string();
        let out = kane::run_testmd(
            &RunOpts {
                kane_bin: &ctx.kane_bin,
                dir: &dir,
                test_file: t,
                timeout: Duration::from_secs(ctx.cfg.timeout_s),
                variables: creds.clone(),
                author: true,
            },
            None,
        )
        .await?;
        let v = judge(&out, None, ctx.cfg.login.is_some());
        rows.push(SuiteRow {
            name,
            kind: v.kind,
            detail: v.headline,
        });
    }
    Ok(rows)
}

pub async fn run(a: SuiteArgs) -> Result<()> {
    let ctx = crate::make_ctx(&a.config)?;
    anyhow::ensure!(
        std::fs::read_dir(&a.dir)?
            .flatten()
            .any(|e| e.file_name().to_string_lossy().ends_with("_test.md")),
        "no *_test.md files in {}",
        a.dir.display()
    );
    let rows = run_dir(&ctx, &a.dir, None).await?;
    println!("| Test | Result | Detail |\n|---|---|---|");
    for r in &rows {
        println!(
            "| {} | {} | {} |",
            r.name,
            r.label(),
            r.detail.replace('|', "\\|")
        );
    }
    if rows.iter().any(|r| r.kind == VerdictKind::Reproduced) {
        std::process::exit(1);
    }
    if rows.iter().any(|r| r.kind == VerdictKind::Inconclusive) {
        std::process::exit(2);
    }
    Ok(())
}
