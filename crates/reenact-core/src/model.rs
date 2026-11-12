//! Shared data types. Everything that crosses a process boundary (CLI -> server,
//! CLI -> GitHub comment) is defined here and is `serde`-serializable.

use serde::{Deserialize, Serialize};

/// A bug report exactly as the reporter wrote it. Treated as untrusted data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub title: String,
    pub body: String,
    pub source_url: Option<String>,
    pub repo: Option<String>,
    pub number: Option<u64>,
}

/// The structured reading of a report. This is the only thing the LLM is
/// allowed to produce; the Kane test file is rendered from it by Rust code.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Plan {
    /// False for crashes in native apps, backend-only issues, feature requests, etc.
    pub in_browser: bool,
    /// Path (and optional query) on the app under test, e.g. `/` or `/settings`.
    pub start_path: String,
    /// State that must exist before the steps (e.g. "a memo exists").
    #[serde(default)]
    pub preconditions: Vec<String>,
    /// Imperative, single-action steps a person would do in the browser.
    #[serde(default)]
    pub steps: Vec<String>,
    /// What the reporter says SHOULD happen, written as a claim about the page.
    #[serde(default)]
    pub expected: String,
    /// What the reporter says actually happens.
    #[serde(default)]
    pub reported_actual: String,
    /// Things a maintainer would have to ask the reporter.
    #[serde(default)]
    pub missing: Vec<String>,
    /// Compiler self-assessment, 0.0 - 1.0.
    #[serde(default)]
    pub confidence: f32,
    /// One-line restatement used as a case title.
    #[serde(default)]
    pub summary: String,
    /// False when the bug lives on a signed-out page (sign-in, sign-up, logged-out state).
    #[serde(default = "yes")]
    pub signed_in: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerdictKind {
    /// Kane carried out the steps and the reported expectation was violated.
    Reproduced,
    /// Kane carried out the steps and the expectation held on this version.
    NotReproduced,
    /// The report is not enough for a browser run, or Kane got stuck before the check.
    NeedsInfo,
    /// Infrastructure problem (Kane error, timeout, credits). Says nothing about the bug.
    Inconclusive,
}

impl VerdictKind {
    pub fn label(self) -> &'static str {
        match self {
            VerdictKind::Reproduced => "Reproduced",
            VerdictKind::NotReproduced => "Not reproduced",
            VerdictKind::NeedsInfo => "Needs info",
            VerdictKind::Inconclusive => "Inconclusive",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    pub kind: VerdictKind,
    /// One or two sentences a maintainer can read without opening anything.
    pub headline: String,
    /// Where Kane stopped when the verdict is `NeedsInfo`.
    pub stuck_at: Option<String>,
    /// Kane's own triage of the failure (root cause), when it produced one.
    pub root_cause: Option<String>,
    pub suggested_fix: Option<String>,
    pub severity: Option<String>,
    pub confidence: Option<f64>,
}

/// One screenshot in the evidence strip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub ordinal: u32,
    pub kind: String,
    pub label: String,
    pub status: String,
    /// File name inside the case directory (jpg/png).
    pub file: String,
    pub annotated_file: Option<String>,
    /// Acted-on element in viewport pixels (x, y, w, h).
    pub rect: Option<[f64; 4]>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Signals {
    pub console_errors: Vec<String>,
    pub failed_requests: Vec<String>,
}

/// What one Kane execution produced, boiled down.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub status: String,
    pub reason: Option<String>,
    pub reason_code: Option<String>,
    pub summary: Option<String>,
    pub credits: f64,
    pub duration_s: f64,
    pub share_url: Option<String>,
    pub failed_step_heading: Option<String>,
    pub bug_confirmed: Option<bool>,
    pub bug_family: Option<String>,
    pub bug_title: Option<String>,
    pub frames: Vec<Frame>,
    /// Evidence strip image for this run, relative to the case folder.
    #[serde(default)]
    pub strip_file: Option<String>,
    pub signals: Signals,
    pub root_cause: Option<String>,
    pub suggested_fix: Option<String>,
    pub severity: Option<String>,
    pub bug_confidence: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunKind {
    Reenact,
    VerifyFix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub kind: RunKind,
    pub label: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub verdict: Verdict,
    pub run: RunSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixAttempt {
    pub attempt: u32,
    pub rationale: String,
    pub files: Vec<String>,
    pub applied: bool,
    pub outcome: Option<VerdictKind>,
    pub note: String,
}

/// The unit that is stored, uploaded and rendered: one report and everything
/// that happened to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub report: Report,
    pub app_url: String,
    pub app_label: Option<String>,
    pub plan: Option<Plan>,
    pub test_md: Option<String>,
    pub runs: Vec<RunRecord>,
    pub fixes: Vec<FixAttempt>,
    pub pr_url: Option<String>,
    pub state: CaseState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseState {
    Queued,
    Compiling,
    Reenacting,
    Reproduced,
    NotReproduced,
    NeedsInfo,
    Inconclusive,
    Fixing,
    Fixed,
    FixFailed,
}

impl From<VerdictKind> for CaseState {
    fn from(v: VerdictKind) -> Self {
        match v {
            VerdictKind::Reproduced => CaseState::Reproduced,
            VerdictKind::NotReproduced => CaseState::NotReproduced,
            VerdictKind::NeedsInfo => CaseState::NeedsInfo,
            VerdictKind::Inconclusive => CaseState::Inconclusive,
        }
    }
}
