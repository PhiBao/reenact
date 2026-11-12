//! Historical benchmark: for each closed bug of a real app, run the SAME compiled
//! plan against the version that had the bug and the version that fixed it.
//! A correct result is Reproduced on the buggy tag and Not reproduced on the
//! fixed tag. Controls (not bugs) must be Not reproduced on both.

use anyhow::{Context, Result};
use clap::Args;
use reenact_core::config::{Config, Login};
use reenact_core::model::*;
use reenact_core::pipeline::{self, Ctx};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

#[derive(Args)]
pub struct BenchArgs {
    #[arg(long, default_value = "benchmark/cases.json")]
    cases: PathBuf,
    #[arg(long, default_value = "benchmark/runs")]
    out: PathBuf,
    /// Only these case ids (comma separated).
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,
    #[arg(long, default_value_t = 2)]
    parallel: usize,
    #[arg(long, default_value = "ghcr.io/usememos/memos")]
    image: String,
    #[arg(long, default_value_t = 15300)]
    base_port: u16,
}

#[derive(Deserialize, Clone)]
struct BenchCase {
    id: String,
    kind: String,
    issue_url: String,
    issue_number: u64,
    title: String,
    body: String,
    buggy_tag: String,
    fixed_tag: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Side {
    tag: String,
    verdict: String,
    headline: String,
    credits: f64,
    duration_s: f64,
    case_id: String,
    correct: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Row {
    id: String,
    kind: String,
    issue_url: String,
    title: String,
    plan_steps: usize,
    plan_expected: String,
    buggy: Option<Side>,
    fixed: Option<Side>,
    error: Option<String>,
}

fn docker(args: &[&str]) -> Result<String> {
    let out = Command::new("docker").args(args).output().context("running docker")?;
    anyhow::ensure!(out.status.success(), "docker {} failed: {}", args.join(" "), String::from_utf8_lossy(&out.stderr));
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

struct Container {
    name: String,
}
impl Drop for Container {
    fn drop(&mut self) {
        let _ = Command::new("docker").args(["rm", "-f", &self.name]).output();
    }
}

async fn start_memos(image: &str, tag: &str, port: u16, label: &str) -> Result<Container> {
    let name = format!("reenact-bench-{label}-{port}");
    let _ = docker(&["rm", "-f", &name]);
    docker(&["run", "-d", "--name", &name, "-p", &format!("{port}:5230"), &format!("{image}:{tag}")])?;
    let c = Container { name };
    let http = reqwest::Client::new();
    let base = format!("http://localhost:{port}");
    let mut ready = false;
    for _ in 0..90 {
        if let Ok(r) = http.get(&base).send().await {
            if r.status().is_success() {
                ready = true;
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    anyhow::ensure!(ready, "memos {tag} did not become ready");
    // First user becomes admin. 0.26+ takes a flat body, 0.25.x the nested one.
    let flat = serde_json::json!({"username":"admin","password":"Passw0rd!"});
    let nested = serde_json::json!({"user":{"username":"admin","password":"Passw0rd!","role":"HOST"}});
    let url = format!("{base}/api/v1/users");
    let r = http.post(&url).json(&flat).send().await?;
    if !r.status().is_success() {
        let r2 = http.post(&url).json(&nested).send().await?;
        anyhow::ensure!(r2.status().is_success(), "could not create the first user on {tag}: {}", r2.status());
    }
    Ok(c)
}

fn cfg_for(port: u16) -> Config {
    Config {
        app_url: format!("http://localhost:{port}"),
        description: "Memos, a self-hosted note-taking app".into(),
        allowed_hosts: vec![],
        login: Some(Login {
            path: "/auth".into(),
            instruction: "Sign in with username {{username}} and password {{password}}.".into(),
            signed_in_check: "the sign-in form is gone and the memo app's main page is shown".into(),
            username_env: "REENACT_USERNAME".into(),
            password_env: "REENACT_PASSWORD".into(),
        }),
        max_steps: 40,
        timeout_s: 420,
    }
}

fn expected_kind(bug: bool, buggy_side: bool) -> VerdictKind {
    if bug && buggy_side {
        VerdictKind::Reproduced
    } else {
        VerdictKind::NotReproduced
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_side(ctx: &Ctx, image: &str, tag: &str, port: u16, label: &str, report: &Report, plan: &Plan, out: &Path, want: VerdictKind) -> Result<Side> {
    let _c = start_memos(image, tag, port, label).await?;
    let mut ctx = ctx.clone();
    ctx.cfg = cfg_for(port);
    let (case, _dir) = pipeline::reenact_with_plan(&ctx, out, report.clone(), plan.clone()).await?;
    let rec = case.runs.last().context("no run")?;
    Ok(Side {
        tag: tag.to_string(),
        verdict: rec.verdict.kind.label().to_string(),
        headline: rec.verdict.headline.clone(),
        credits: rec.run.credits,
        duration_s: rec.run.duration_s,
        case_id: case.id.clone(),
        correct: rec.verdict.kind == want,
    })
}

pub async fn run(a: BenchArgs) -> Result<()> {
    std::env::set_var("REENACT_USERNAME", "admin");
    std::env::set_var("REENACT_PASSWORD", "Passw0rd!");
    let raw = std::fs::read_to_string(&a.cases)?;
    let all: Vec<BenchCase> = serde_json::from_str(&raw)?;
    let cases: Vec<BenchCase> = all.into_iter().filter(|c| a.only.is_empty() || a.only.contains(&c.id)).collect();

    // Pull each tag once, up front.
    let mut tags: Vec<String> = cases.iter().flat_map(|c| [c.buggy_tag.clone(), c.fixed_tag.clone()]).collect();
    tags.sort();
    tags.dedup();
    for t in &tags {
        eprintln!("pulling {}:{t}", a.image);
        docker(&["pull", "-q", &format!("{}:{t}", a.image)])?;
    }

    let base_ctx = crate::make_ctx(&PathBuf::from("benchmark/bench.reenact.yml"))?;
    let sem = Arc::new(Semaphore::new(a.parallel.max(1)));
    let mut handles = Vec::new();
    for (i, c) in cases.into_iter().enumerate() {
        let sem = sem.clone();
        let ctx = base_ctx.clone();
        let out = a.out.join(&c.id);
        let image = a.image.clone();
        let port = a.base_port + (i as u16) * 2;
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire_owned().await.unwrap();
            let report = Report {
                title: c.title.clone(),
                body: c.body.clone(),
                source_url: Some(c.issue_url.clone()),
                repo: None,
                number: Some(c.issue_number),
            };
            let mut row = Row {
                id: c.id.clone(),
                kind: c.kind.clone(),
                issue_url: c.issue_url.clone(),
                title: c.title.clone(),
                plan_steps: 0,
                plan_expected: String::new(),
                buggy: None,
                fixed: None,
                error: None,
            };
            let res: Result<()> = async {
                let mut pctx = ctx.clone();
                pctx.cfg = cfg_for(port);
                let plan = pipeline::prepare(&pctx, &report).await?;
                row.plan_steps = plan.steps.len();
                row.plan_expected = plan.expected.clone();
                let is_bug = c.kind == "bug";
                row.buggy = Some(run_side(&ctx, &image, &c.buggy_tag, port, &format!("{}-b", c.id), &report, &plan, &out.join("buggy"), expected_kind(is_bug, true)).await?);
                row.fixed = Some(run_side(&ctx, &image, &c.fixed_tag, port + 1, &format!("{}-f", c.id), &report, &plan, &out.join("fixed"), expected_kind(is_bug, false)).await?);
                Ok(())
            }
            .await;
            if let Err(e) = res {
                row.error = Some(format!("{e:#}"));
            }
            eprintln!(
                "== {} buggy={} fixed={}",
                row.id,
                row.buggy.as_ref().map(|s| s.verdict.as_str()).unwrap_or("-"),
                row.fixed.as_ref().map(|s| s.verdict.as_str()).unwrap_or("-")
            );
            row
        }));
    }
    let mut rows = Vec::new();
    for h in handles {
        rows.push(h.await?);
    }
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    std::fs::create_dir_all(&a.out)?;
    std::fs::write(a.out.join("results.json"), serde_json::to_vec_pretty(&rows)?)?;
    std::fs::write(a.out.join("RESULTS.md"), render_table(&rows))?;
    println!("{}", render_table(&rows));
    Ok(())
}

fn render_table(rows: &[Row]) -> String {
    let mut s = String::from("| Case | Kind | Buggy version | Fixed version | Correct |\n|---|---|---|---|---|\n");
    let (mut ok, mut total) = (0, 0);
    for r in rows {
        let cell = |x: &Option<Side>| x.as_ref().map(|s| format!("{} ({})", s.verdict, s.tag)).unwrap_or_else(|| "error".into());
        let correct = match (&r.buggy, &r.fixed) {
            (Some(b), Some(f)) => b.correct && f.correct,
            _ => false,
        };
        total += 1;
        if correct {
            ok += 1;
        }
        s.push_str(&format!("| [{}]({}) | {} | {} | {} | {} |\n", r.id, r.issue_url, r.kind, cell(&r.buggy), cell(&r.fixed), if correct { "yes" } else { "no" }));
    }
    s.push_str(&format!("\n{ok} of {total} cases judged correctly on both versions.\n"));
    s
}
