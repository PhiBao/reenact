//! `reenact sweep`: triage a project's open bug reports against a running instance,
//! read-only. Nothing is posted to the project. The result is a table you can hand to a
//! maintainer: which open reports still reproduce on this version, which do not, which
//! need more information.

use anyhow::{Context, Result};
use clap::Args;
use reenact_core::model::*;
use reenact_core::pipeline::{self, Ctx};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Args)]
pub struct SweepArgs {
    #[arg(long, default_value = ".reenact.yml")]
    config: PathBuf,
    /// owner/repo whose open issues are triaged.
    #[arg(long)]
    repo: String,
    /// Only issues with this label.
    #[arg(long, default_value = "bug")]
    label: String,
    /// How many issues to try, newest first.
    #[arg(long, default_value_t = 10)]
    limit: usize,
    /// Skip issues created before this version window (by issue number).
    #[arg(long)]
    min_number: Option<u64>,
    #[arg(long, default_value_t = 1)]
    parallel: usize,
    #[arg(long, default_value = "benchmark/sweep")]
    out: PathBuf,
    /// Label for the version under test, shown in the table.
    #[arg(long)]
    version: Option<String>,
    #[command(flatten)]
    ship: crate::ship::ShipArgs,
}

#[derive(Serialize, Clone)]
struct Row {
    number: u64,
    title: String,
    url: String,
    verdict: String,
    headline: String,
    case_id: String,
    credits: f64,
}

fn open_issues(a: &SweepArgs) -> Result<Vec<Report>> {
    let out = Command::new("gh")
        .args([
            "api",
            &format!(
                "repos/{}/issues?state=open&labels={}&per_page=100&sort=created&direction=desc",
                a.repo, a.label
            ),
        ])
        .output()
        .context("gh api")?;
    anyhow::ensure!(
        out.status.success(),
        "gh failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout)?;
    let mut reports = Vec::new();
    for i in v.as_array().into_iter().flatten() {
        if i.get("pull_request").is_some() {
            continue;
        }
        let number = i["number"].as_u64().unwrap_or(0);
        if a.min_number.is_some_and(|m| number < m) {
            continue;
        }
        reports.push(Report {
            title: i["title"].as_str().unwrap_or_default().to_string(),
            body: i["body"].as_str().unwrap_or_default().to_string(),
            source_url: i["html_url"].as_str().map(str::to_string),
            repo: Some(a.repo.clone()),
            number: Some(number),
        });
        if reports.len() >= a.limit {
            break;
        }
    }
    Ok(reports)
}

pub async fn run(a: SweepArgs) -> Result<()> {
    let ctx = crate::make_ctx(&a.config)?;
    let reports = open_issues(&a)?;
    anyhow::ensure!(
        !reports.is_empty(),
        "no open issues labelled {} in {}",
        a.label,
        a.repo
    );
    eprintln!("sweeping {} open issues of {}", reports.len(), a.repo);

    let sem = Arc::new(Semaphore::new(a.parallel.max(1)));
    let mut handles = Vec::new();
    for r in reports {
        let sem = sem.clone();
        let mut ctx: Ctx = ctx.clone();
        let out = a.out.clone();
        handles.push(tokio::spawn(async move {
            let _p = sem.acquire_owned().await.unwrap();
            let n = r.number.unwrap_or(0);
            ctx.case_id = Some(format!("sweep-{n}"));
            let url = r.source_url.clone().unwrap_or_default();
            let title = r.title.clone();
            match pipeline::reenact(&ctx, &out, r).await {
                Ok((case, _dir)) => {
                    let rec = case.runs.iter().rev().find(|x| x.kind == RunKind::Reenact);
                    Row {
                        number: n,
                        title,
                        url,
                        verdict: rec
                            .map(|x| x.verdict.kind.label().to_string())
                            .unwrap_or_default(),
                        headline: rec.map(|x| x.verdict.headline.clone()).unwrap_or_default(),
                        case_id: case.id,
                        credits: case.runs.iter().map(|x| x.run.credits).sum(),
                    }
                }
                Err(e) => Row {
                    number: n,
                    title,
                    url,
                    verdict: "Inconclusive".into(),
                    headline: format!("{e:#}").chars().take(160).collect(),
                    case_id: String::new(),
                    credits: 0.0,
                },
            }
        }));
    }
    let mut rows = Vec::new();
    for h in handles {
        rows.push(h.await?);
    }
    rows.sort_by_key(|r| std::cmp::Reverse(r.number));

    // Upload each case so the table can link to its case file (nothing is posted to the project).
    if let (Some(server), Some(tok)) = (&a.ship.upload_url, &a.ship.upload_token) {
        for r in rows.iter().filter(|r| !r.case_id.is_empty()) {
            let dir = a.out.join(&r.case_id);
            if let Err(e) = crate::ship::upload_bundle(&dir, &r.case_id, server, tok).await {
                eprintln!("could not upload {}: {e:#}", r.case_id);
            }
        }
    }

    let version = a
        .version
        .clone()
        .unwrap_or_else(|| "the instance under test".into());
    let mut md = format!(
        "# Open `{label}` issues of {repo}, triaged on {version}\n\nRead-only: nothing was posted to the project. Each row is one Kane run (two if the first did not reproduce).\n\n| Issue | Verdict | What Kane saw |\n|---|---|---|\n",
        label = a.label,
        repo = a.repo
    );
    for r in &rows {
        md.push_str(&format!(
            "| [#{}]({}) {} | {} | {} |\n",
            r.number,
            r.url,
            r.title.replace('|', "\\|"),
            r.verdict,
            r.headline.replace('|', "\\|").replace('\n', " ")
        ));
    }
    let count = |l: &str| rows.iter().filter(|r| r.verdict == l).count();
    md.push_str(&format!(
        "\n{} reproduced, {} not reproduced, {} need info, {} inconclusive. About {:.0} Kane credits.\n",
        count("Reproduced"),
        count("Not reproduced"),
        count("Needs info"),
        count("Inconclusive"),
        rows.iter().map(|r| r.credits).sum::<f64>()
    ));
    std::fs::create_dir_all(&a.out)?;
    std::fs::write(a.out.join("SWEEP.md"), &md)?;
    std::fs::write(a.out.join("sweep.json"), serde_json::to_vec_pretty(&rows)?)?;
    println!("{md}");
    Ok(())
}
