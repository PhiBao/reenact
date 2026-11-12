//! `reenact suite`: re-run every saved reproduction against the live app.
//! A test that used to be fixed and now fails with a confirmed product bug is a
//! regression, and the suite exits non-zero.

use anyhow::Result;
use clap::Args;
use reenact_core::kane::{self, RunOpts};
use reenact_core::model::VerdictKind;
use reenact_core::verdict::judge;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Args)]
pub struct SuiteArgs {
    #[arg(long, default_value = ".reenact.yml")]
    config: PathBuf,
    #[arg(long, default_value = "reenact/tests")]
    dir: PathBuf,
}

pub async fn run(a: SuiteArgs) -> Result<()> {
    let a = SuiteArgs { dir: std::fs::canonicalize(&a.dir)?, ..a };
    let ctx = crate::make_ctx(&a.config)?;
    let mut tests: Vec<PathBuf> = std::fs::read_dir(&a.dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with("_test.md")))
        .collect();
    tests.sort();
    anyhow::ensure!(!tests.is_empty(), "no *_test.md files in {}", a.dir.display());

    let creds = match &ctx.cfg.login {
        Some(l) => vec![
            ("username".to_string(), std::env::var(&l.username_env)?, false),
            ("password".to_string(), std::env::var(&l.password_env)?, true),
        ],
        None => vec![],
    };

    let (mut regressed, mut inconclusive) = (0, 0);
    println!("| Test | Result | Detail |\n|---|---|---|");
    for t in &tests {
        let name = t.file_name().unwrap().to_string_lossy().to_string();
        let out = kane::run_testmd(
            &RunOpts {
                kane_bin: &ctx.kane_bin,
                dir: &a.dir,
                test_file: t,
                timeout: Duration::from_secs(ctx.cfg.timeout_s),
                variables: creds.clone(),
                author: true,
            },
            None,
        )
        .await?;
        let v = judge(&out, None, ctx.cfg.login.is_some());
        let label = match v.kind {
            VerdictKind::NotReproduced => "holds",
            VerdictKind::Reproduced => {
                regressed += 1;
                "REGRESSED"
            }
            VerdictKind::NeedsInfo => "unsure",
            VerdictKind::Inconclusive => {
                inconclusive += 1;
                "inconclusive"
            }
        };
        println!("| {name} | {label} | {} |", v.headline.replace('|', "\\|"));
    }
    if regressed > 0 {
        std::process::exit(1);
    }
    if inconclusive > 0 {
        std::process::exit(2);
    }
    Ok(())
}
