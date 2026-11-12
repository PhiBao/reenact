//! Drives `kane-cli testmd run --agent` and folds its NDJSON stream into a
//! `KaneOutcome`.
//!
//! Lessons baked in (see the GuardianKane write-up and our own spikes):
//! * Do not trust the process exit code alone. The verdict comes from the
//!   structured events (`run_end`, `test_md_bug_verdict`).
//! * Killing the child is not enough. Kane spawns Chrome under it, so the child
//!   leads its own process group and we SIGKILL the whole group afterwards.
//! * Credentials go through a 0600 variables file, never the command line.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Default)]
pub struct BugVerdict {
    pub confirmed: bool,
    pub title: Option<String>,
    pub family: Option<String>,
    pub category: Option<String>,
    pub severity: Option<String>,
    pub confidence: Option<f64>,
    pub one_liner: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StepOutcome {
    pub index: u32,
    pub heading: String,
    pub status: String,
    pub reason: Option<String>,
    pub reason_code: Option<String>,
    pub summary: Option<String>,
    pub credits: f64,
    pub duration_s: f64,
    pub bug: Option<BugVerdict>,
}

#[derive(Debug, Clone, Default)]
pub struct KaneOutcome {
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub steps: Vec<StepOutcome>,
    pub overall_status: Option<String>,
    pub duration_s: f64,
    pub share_url: Option<String>,
    pub evidence_id: Option<String>,
    pub evidence_pack: Option<PathBuf>,
    pub stderr_tail: String,
    /// First error-looking event or line, for `Inconclusive` explanations.
    pub error: Option<String>,
    pub saw_stream_start: bool,
}

impl KaneOutcome {
    pub fn total_credits(&self) -> f64 {
        self.steps.iter().map(|s| s.credits).sum()
    }
}

/// Incremental folder for the event stream. Also used by tests on recorded runs.
#[derive(Default)]
pub struct Acc {
    pub out: KaneOutcome,
    steps: BTreeMap<u32, StepOutcome>,
}

impl Acc {
    pub fn feed(&mut self, ev: &Value) {
        let ty = ev["type"].as_str().unwrap_or_default();
        match ty {
            "stream_start" => self.out.saw_stream_start = true,
            "test_md_step_start" => {
                let idx = ev["step_index"].as_u64().unwrap_or(0) as u32;
                let e = self.steps.entry(idx).or_default();
                e.index = idx;
                e.heading = ev["heading"].as_str().unwrap_or_default().to_string();
                e.status = "running".into();
            }
            "run_end" => {
                // run_end carries no step index; attribute to the latest started step.
                if let Some((_, s)) = self.steps.iter_mut().next_back() {
                    s.reason = ev["reason"].as_str().map(str::to_string);
                    s.reason_code = ev["reason_code"].as_str().map(str::to_string);
                    s.summary = ev["summary"].as_str().map(str::to_string);
                    s.credits += ev["credits_consumed"].as_f64().unwrap_or(0.0);
                    if s.status == "running" {
                        s.status = ev["status"].as_str().unwrap_or("").to_string();
                    }
                }
            }
            "test_md_step_end" => {
                let idx = ev["step_index"].as_u64().unwrap_or(0) as u32;
                let e = self.steps.entry(idx).or_default();
                e.index = idx;
                e.status = ev["status"].as_str().unwrap_or("").to_string();
                e.duration_s = ev["duration_s"].as_f64().unwrap_or(0.0);
            }
            "test_md_bug_verdict" => {
                let idx = ev["step_index"].as_u64().unwrap_or(0) as u32;
                let e = self.steps.entry(idx).or_default();
                e.index = idx;
                e.bug = Some(BugVerdict {
                    confirmed: ev["confirmed"].as_bool().unwrap_or(false),
                    title: ev["bug_title"].as_str().map(str::to_string),
                    family: ev["family"].as_str().map(str::to_string),
                    category: ev["category"].as_str().map(str::to_string),
                    severity: ev["severity"].as_str().map(str::to_string),
                    confidence: ev["confidence"].as_f64(),
                    one_liner: ev["one_liner"].as_str().map(str::to_string),
                });
            }
            "test_md_evidence_ingest" => {
                if let Some(id) = ev["evidence_id"].as_str() {
                    self.out.evidence_id = Some(id.to_string());
                }
            }
            "test_md_summary" | "test_md_done" => {
                if let Some(s) = ev["overall_status"].as_str() {
                    self.out.overall_status = Some(s.to_string());
                }
                if let Some(d) = ev["duration_s"].as_f64() {
                    self.out.duration_s = d;
                }
                if let Some(u) = ev["share_url"].as_str() {
                    self.out.share_url = Some(u.to_string());
                }
            }
            t if (t.contains("error") || t.contains("fatal")) && self.out.error.is_none() => {
                let msg = ev["message"]
                    .as_str()
                    .or_else(|| ev["error"].as_str())
                    .unwrap_or(t);
                self.out.error = Some(msg.chars().take(300).collect());
            }
            _ => {}
        }
    }

    pub fn finish(mut self) -> KaneOutcome {
        self.out.steps = self.steps.into_values().collect();
        self.out
    }
}

/// Fold a whole recorded stream. Handy for tests and for re-judging stored runs.
pub fn parse_stream(ndjson: &str) -> KaneOutcome {
    let mut acc = Acc::default();
    for line in ndjson.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            acc.feed(&v);
        }
    }
    acc.finish()
}

pub struct RunOpts<'a> {
    pub kane_bin: &'a str,
    /// Working directory. The test file must live here; evidence lands in `<dir>/.testmuai/evidence`.
    pub dir: &'a Path,
    pub test_file: &'a Path,
    pub timeout: Duration,
    /// Secret variables, e.g. username/password. Written to a 0600 temp file.
    pub variables: Vec<(String, String, bool)>,
    /// Re-author every step with the live agent instead of replaying the cache.
    /// Cached replays reuse recorded check results, so they cannot catch a
    /// regression; anything that must be judged against the live app uses this.
    pub author: bool,
}

pub type OnEvent<'a> = &'a (dyn Fn(&Value) + Send + Sync);

/// SIGKILL a whole process group. Safe to call on a dead group.
pub fn kill_group(pid: u32) {
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }
}

pub async fn run_testmd(opts: &RunOpts<'_>, on_event: Option<OnEvent<'_>>) -> Result<KaneOutcome> {
    anyhow::ensure!(
        opts.dir.is_absolute() && opts.test_file.is_absolute(),
        "kane paths must be absolute"
    );
    let mut args: Vec<String> = vec![
        "testmd".into(),
        "run".into(),
        opts.test_file.to_string_lossy().into_owned(),
        "--agent".into(),
        "--headless".into(),
        "--bug-detection".into(),
        "stop".into(),
    ];

    if opts.author {
        args.push("--author".into());
    }

    // Credentials: 0600 file, removed when dropped.
    let var_file = if opts.variables.is_empty() {
        None
    } else {
        let mut obj = serde_json::Map::new();
        for (k, v, secret) in &opts.variables {
            obj.insert(k.clone(), serde_json::json!({"value": v, "secret": secret}));
        }
        let f = tempfile::Builder::new()
            .prefix("reenact-vars-")
            .suffix(".json")
            .tempfile()?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(f.path(), std::fs::Permissions::from_mode(0o600))?;
        std::fs::write(f.path(), serde_json::to_vec(&Value::Object(obj))?)?;
        args.push("--variables-file".into());
        args.push(f.path().to_string_lossy().into_owned());
        Some(f)
    };

    let mut cmd = Command::new(opts.kane_bin);
    cmd.args(&args)
        .current_dir(opts.dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true);
    let mut child = cmd
        .spawn()
        .with_context(|| format!("spawning {}", opts.kane_bin))?;
    let pid = child.id().unwrap_or(0);

    let stdout = child.stdout.take().context("no stdout")?;
    let mut stderr = child.stderr.take().context("no stderr")?;
    let stderr_task = tokio::spawn(async move {
        let mut buf = String::new();
        let _ = stderr.read_to_string(&mut buf).await;
        buf
    });

    let mut acc = Acc::default();
    let mut lines = BufReader::new(stdout).lines();
    let mut timed_out = false;

    let read_all = async {
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(v) = serde_json::from_str::<Value>(&line) {
                if let Some(cb) = on_event {
                    cb(&v);
                }
                acc.feed(&v);
            }
        }
    };

    if tokio::time::timeout(opts.timeout, read_all).await.is_err() {
        timed_out = true;
    }
    if pid != 0 {
        kill_group(pid);
    }
    let status = child.wait().await.ok();
    let stderr_text = stderr_task.await.unwrap_or_default();
    drop(var_file);

    let mut out = acc.finish();
    out.timed_out = timed_out;
    out.exit_code = status.and_then(|s| s.code());
    out.stderr_tail = stderr_text
        .chars()
        .rev()
        .take(1500)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    // Locate the sealed evidence pack.
    let ev_dir = opts.dir.join(".testmuai").join("evidence");
    if let Some(id) = &out.evidence_id {
        let p = ev_dir.join(format!("{id}.evidence"));
        if p.exists() {
            out.evidence_pack = Some(p);
        }
    }
    if out.evidence_pack.is_none() {
        out.evidence_pack = newest_evidence(&ev_dir);
    }
    Ok(out)
}

fn newest_evidence(dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for e in std::fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("evidence") {
            continue;
        }
        let m = e.metadata().ok()?.modified().ok()?;
        if best.as_ref().is_none_or(|(t, _)| m > *t) {
            best = Some((m, p));
        }
    }
    best.map(|(_, p)| p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_recorded_failing_run() {
        let raw = include_str!("../tests/fixtures/coupon-bug.ndjson");
        let o = parse_stream(raw);
        assert!(o.saw_stream_start);
        assert_eq!(o.overall_status.as_deref(), Some("failed"));
        assert_eq!(o.steps.len(), 1);
        let s = &o.steps[0];
        assert_eq!(s.heading, "Reproduce reported bug");
        assert_eq!(s.status, "failed");
        assert_eq!(
            s.reason_code.as_deref(),
            Some("assertion_error.confirmed_product_bug")
        );
        let b = s.bug.as_ref().unwrap();
        assert!(b.confirmed);
        assert_eq!(b.family.as_deref(), Some("application_issue"));
        assert!(s.credits > 10.0);
        assert!(o.evidence_id.is_some());
        assert!(o.share_url.is_some());
    }
}
