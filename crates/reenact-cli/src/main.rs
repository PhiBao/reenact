use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reenact_core::comment::{render_comment, Links};
use reenact_core::config::Config;
use reenact_core::github;
use reenact_core::llm::Llm;
use reenact_core::model::*;
use reenact_core::pipeline::{self, Ctx};
use std::path::PathBuf;
use std::sync::Arc;

mod bench;
mod fix;
mod ship;
mod suite;

#[derive(Parser)]
#[command(name = "reenact", version, about = "Bug report in, browser reproduction and proof out.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Reproduce a bug report in a real browser with Kane.
    Repro {
        #[arg(long, default_value = ".reenact.yml")]
        config: PathBuf,
        /// GitHub issue URL to read the report from.
        #[arg(long, conflicts_with_all = ["title", "body_file"])]
        issue: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        body_file: Option<PathBuf>,
        #[arg(long, default_value = ".reenact/cases")]
        out: PathBuf,
        /// Use this case id (CI uses `issue-<n>`). Letters, digits, `-`, `_`.
        #[arg(long)]
        case_id: Option<String>,
        #[command(flatten)]
        ship: ship::ShipArgs,
    },
    /// Re-run a case's saved test against the app as it is now.
    Verify {
        #[arg(long, default_value = ".reenact.yml")]
        config: PathBuf,
        #[arg(long)]
        case: PathBuf,
    },
    /// Print the GitHub comment for a case.
    Comment {
        #[arg(long)]
        case: PathBuf,
        #[arg(long)]
        strip_url: Option<String>,
        #[arg(long)]
        compare_url: Option<String>,
        #[arg(long)]
        case_url: Option<String>,
    },
    /// Attempt a fix with an agent, verified by the case's own test.
    Fix(fix::FixArgs),
    /// Re-run every saved reproduction live; fail if a fixed bug is back.
    Suite(suite::SuiteArgs),
    /// Run the historical benchmark (buggy and fixed versions of a real app).
    Bench(bench::BenchArgs),
}

pub fn make_ctx(config: &PathBuf) -> Result<Ctx> {
    let cfg = Config::load(config)?;
    let llm = Llm::from_env().ok();
    let progress: pipeline::Progress = Arc::new(|stage, msg| eprintln!("[{stage}] {msg}"));
    Ok(Ctx { cfg, llm, kane_bin: std::env::var("KANE_CLI_BIN").unwrap_or_else(|_| "kane-cli".into()), progress, case_id: None })
}

fn short_summary(case: &Case) -> serde_json::Value {
    let last = case.runs.last();
    serde_json::json!({
        "case": case.id,
        "state": case.state,
        "verdict": last.map(|r| r.verdict.kind.label()),
        "headline": last.map(|r| r.verdict.headline.clone()),
        "credits": case.runs.iter().map(|r| r.run.credits).sum::<f64>(),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()))
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Repro { config, issue, title, body_file, out, case_id, ship } => {
            let mut ctx = make_ctx(&config)?;
            if let Some(id) = case_id {
                anyhow::ensure!(!id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'), "bad --case-id");
                ctx.case_id = Some(id);
            }
            let report = match (issue, title, body_file) {
                (Some(url), _, _) => github::fetch_issue(&url)?,
                (None, Some(t), Some(f)) => Report {
                    title: t,
                    body: std::fs::read_to_string(&f).with_context(|| format!("reading {}", f.display()))?,
                    source_url: None,
                    repo: None,
                    number: None,
                },
                _ => anyhow::bail!("give --issue <url>, or --title and --body-file"),
            };
            let (case, dir) = pipeline::reenact(&ctx, &out, report).await?;
            println!("{}", serde_json::to_string_pretty(&short_summary(&case))?);
            eprintln!("case directory: {}", dir.display());
            ship::ship(&dir, &ship).await?;
        }
        Cmd::Verify { config, case } => {
            let ctx = make_ctx(&config)?;
            let rec = pipeline::verify(&ctx, &case, "verify").await?;
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "verdict": rec.verdict.kind.label(), "headline": rec.verdict.headline, "credits": rec.run.credits,
            }))?);
        }
        Cmd::Comment { case, strip_url, compare_url, case_url } => {
            let c = pipeline::load_case(&case)?;
            println!("{}", render_comment(&c, &Links { strip_url: strip_url.as_deref(), compare_url: compare_url.as_deref(), case_url: case_url.as_deref() }));
        }
        Cmd::Fix(a) => fix::run(a).await?,
        Cmd::Suite(a) => suite::run(a).await?,
        Cmd::Bench(a) => bench::run(a).await?,
    }
    Ok(())
}
