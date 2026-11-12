//! The end-to-end flow for one report: compile -> render -> run Kane -> read
//! evidence -> judge -> compose proof. Also the re-verification used by the fix loop.

use crate::compile::{compile_report, plan_gap};
use crate::config::Config;
use crate::evidence::{self, Extracted};
use crate::kane::{self, KaneOutcome, RunOpts};
use crate::llm::Llm;
use crate::model::*;
use crate::render::render_test_md;
use crate::strip;
use crate::verdict::{judge, summarize};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub type Progress = Arc<dyn Fn(&str, &str) + Send + Sync>;

pub fn no_progress() -> Progress {
    Arc::new(|_, _| {})
}

#[derive(Clone)]
pub struct Ctx {
    pub cfg: Config,
    pub llm: Option<Llm>,
    pub kane_bin: String,
    pub progress: Progress,
    /// Fixed case id (CI uses `issue-<n>` so a case is addressable from the issue).
    pub case_id: Option<String>,
}

impl Ctx {
    fn say(&self, stage: &str, msg: &str) {
        (self.progress)(stage, msg);
    }

    fn credentials(&self) -> Result<Vec<(String, String, bool)>> {
        let Some(login) = &self.cfg.login else { return Ok(vec![]) };
        let u = std::env::var(&login.username_env).with_context(|| format!("{} is not set", login.username_env))?;
        let p = std::env::var(&login.password_env).with_context(|| format!("{} is not set", login.password_env))?;
        Ok(vec![("username".into(), u, false), ("password".into(), p, true)])
    }
}

pub fn new_case_id() -> String {
    format!("c-{}", &uuid::Uuid::new_v4().simple().to_string()[..8])
}

pub fn case_dir(base: &Path, id: &str) -> PathBuf {
    base.join(id)
}

pub fn save_case(dir: &Path, case: &Case) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("case.json"), serde_json::to_vec_pretty(case)?)?;
    Ok(())
}

pub fn load_case(dir: &Path) -> Result<Case> {
    let raw = std::fs::read(dir.join("case.json")).with_context(|| format!("reading {}/case.json", dir.display()))?;
    Ok(serde_json::from_slice(&raw)?)
}

/// Forward interesting Kane events as human-readable progress lines.
fn kane_event_to_progress(ctx: &Ctx, ev: &Value) {
    match ev["type"].as_str().unwrap_or_default() {
        "test_md_step_start" => ctx.say("step", &format!("Step: {}", ev["heading"].as_str().unwrap_or(""))),
        "step_event" if ev["event"] == "action" => {
            ctx.say("action", ev["detail"].as_str().unwrap_or(""));
        }
        "test_md_bug_verdict" => ctx.say(
            "triage",
            &format!(
                "Kane triage: {}",
                ev["one_liner"].as_str().or_else(|| ev["bug_title"].as_str()).unwrap_or("")
            ),
        ),
        _ => {}
    }
}

/// Run `test_md` once in `dir`, read the evidence, and produce a `RunRecord`.
/// `prefix` namespaces frame files so repro and verify frames can coexist.
pub async fn run_and_record(
    ctx: &Ctx,
    dir: &Path,
    test_md: &str,
    kind: RunKind,
    label: &str,
    prefix: &str,
) -> Result<(RunRecord, Option<(u32, u32)>)> {
    std::fs::create_dir_all(dir)?;
    // Kane runs with this directory as its cwd, so everything must be absolute.
    let dir = &std::fs::canonicalize(dir)?;
    let test_file = dir.join("reenact_test.md");
    std::fs::write(&test_file, test_md)?;
    let started_at = chrono::Utc::now();

    let creds = ctx.credentials()?;
    let opts = RunOpts {
        kane_bin: &ctx.kane_bin,
        dir,
        test_file: &test_file,
        timeout: Duration::from_secs(ctx.cfg.timeout_s),
        variables: creds,
        author: true,
    };
    ctx.say("running", "Kane is driving a real browser");
    let cb = |ev: &Value| kane_event_to_progress(ctx, ev);
    let outcome: KaneOutcome = kane::run_testmd(&opts, Some(&cb)).await?;

    let frames_dir = dir.join("frames");
    let extracted: Option<Extracted> = match &outcome.evidence_pack {
        Some(p) => match evidence::extract(p, &frames_dir, prefix) {
            Ok(e) => Some(e),
            Err(err) => {
                tracing::warn!("could not read evidence pack: {err:#}");
                None
            }
        },
        None => None,
    };
    let viewport = extracted.as_ref().and_then(|e| e.viewport);
    let verdict = judge(&outcome, extracted.as_ref(), ctx.cfg.login.is_some());
    let mut run = summarize(&outcome, extracted.as_ref(), None);
    // Frame files are relative to `frames/`; store them with that prefix so a
    // case directory is self-describing.
    for f in &mut run.frames {
        f.file = format!("frames/{}", f.file);
        if let Some(a) = &mut f.annotated_file {
            *a = format!("frames/{a}");
        }
    }

    // Strip for this run.
    if !run.frames.is_empty() {
        let strip_name = format!("{prefix}strip.png");
        // Leave the sign-in screens out of the strip: they say nothing about the bug.
        let login_path = ctx.cfg.login.as_ref().map(|l| l.path.clone());
        let mut shown: Vec<Frame> = run
            .frames
            .iter()
            .filter(|f| match (&login_path, &f.url) {
                (Some(lp), Some(u)) => !url::Url::parse(u).map(|u| u.path() == lp).unwrap_or(false),
                _ => true,
            })
            .cloned()
            .collect();
        if shown.len() < 2 {
            shown = run.frames.clone();
        }
        if let Err(e) = strip::compose_strip(dir, &shown, verdict.kind, viewport.unwrap_or((1920, 1080)), &dir.join(&strip_name)) {
            tracing::warn!("strip failed: {e:#}");
        }
    }
    ctx.say("verdict", &format!("{}: {}", verdict.kind.label(), verdict.headline));
    Ok((RunRecord { kind, label: label.to_string(), started_at, verdict, run }, viewport))
}

/// Compile a report into a sanitized plan (one LLM call, no browser).
pub async fn prepare(ctx: &Ctx, report: &Report) -> Result<Plan> {
    ctx.say("compiling", "Reading the report");
    let llm = ctx.llm.as_ref().context("an LLM is required to compile a report")?;
    let plan = compile_report(llm, &ctx.cfg, report).await?;
    ctx.say("compiled", &format!("{} steps, confidence {:.0}%", plan.steps.len(), plan.confidence * 100.0));
    Ok(plan)
}

/// Report -> Case. `base` is the cases directory; the case gets its own subfolder.
pub async fn reenact(ctx: &Ctx, base: &Path, report: Report) -> Result<(Case, PathBuf)> {
    let plan = prepare(ctx, &report).await?;
    reenact_with_plan(ctx, base, report, plan).await
}

/// Run an already-compiled plan. The benchmark uses this to run the SAME plan
/// against the buggy and the fixed version of an app.
pub async fn reenact_with_plan(ctx: &Ctx, base: &Path, report: Report, plan: Plan) -> Result<(Case, PathBuf)> {
    let id = ctx.case_id.clone().unwrap_or_else(new_case_id);
    let dir = case_dir(base, &id);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    let mut case = Case {
        id: id.clone(),
        created_at: chrono::Utc::now(),
        report: report.clone(),
        app_url: ctx.cfg.app_url.clone(),
        app_label: (!ctx.cfg.description.is_empty()).then(|| ctx.cfg.description.clone()),
        plan: Some(plan.clone()),
        test_md: None,
        runs: vec![],
        fixes: vec![],
        pr_url: None,
        state: CaseState::Compiling,
    };

    if let Some(gap) = plan_gap(&plan) {
        // No browser run: say why, and what to ask.
        let verdict = Verdict {
            kind: VerdictKind::NeedsInfo,
            headline: gap,
            stuck_at: None,
            root_cause: None,
            suggested_fix: None,
            severity: None,
            confidence: None,
        };
        case.runs.push(RunRecord {
            kind: RunKind::Reenact,
            label: "intake".into(),
            started_at: chrono::Utc::now(),
            verdict: verdict.clone(),
            run: empty_run(),
        });
        case.state = CaseState::NeedsInfo;
        save_case(&dir, &case)?;
        ctx.say("verdict", &format!("{}: {}", verdict.kind.label(), verdict.headline));
        return Ok((case, dir));
    }

    let tag = report.number.map(|n| format!("issue-{n}")).unwrap_or_else(|| id.clone());
    let test_md = render_test_md(&ctx.cfg, &report, &plan, &tag);
    case.test_md = Some(test_md.clone());
    case.state = CaseState::Reenacting;
    save_case(&dir, &case)?;

    let (rec, _) = run_and_record(ctx, &dir, &test_md, RunKind::Reenact, "reenact", "").await?;
    case.state = rec.verdict.kind.into();
    case.runs.push(rec);
    save_case(&dir, &case)?;
    Ok((case, dir))
}

/// Re-run a case's saved test against the current app (after a fix).
pub async fn verify(ctx: &Ctx, dir: &Path, label: &str) -> Result<RunRecord> {
    let mut case = load_case(dir)?;
    let md = case.test_md.clone().context("case has no saved test")?;
    let n = case.runs.iter().filter(|r| r.kind == RunKind::VerifyFix).count() + 1;
    let prefix = format!("fix{n}-");
    let (rec, _) = run_and_record(ctx, dir, &md, RunKind::VerifyFix, label, &prefix).await?;
    case.runs.push(rec.clone());
    save_case(dir, &case)?;
    Ok(rec)
}

pub fn empty_run() -> RunSummary {
    RunSummary {
        exit_code: None,
        timed_out: false,
        status: "not_run".into(),
        reason: None,
        reason_code: None,
        summary: None,
        credits: 0.0,
        duration_s: 0.0,
        share_url: None,
        failed_step_heading: None,
        bug_confirmed: None,
        bug_family: None,
        bug_title: None,
        frames: vec![],
        signals: Signals::default(),
        root_cause: None,
        suggested_fix: None,
        severity: None,
        bug_confidence: None,
    }
}
