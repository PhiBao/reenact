//! Turns what Kane reported into one of four verdicts a maintainer can act on.
//!
//! The rule that matters: `Reproduced` requires BOTH that the check at the end
//! of the reproduction step failed AND that Kane's own investigation classed the
//! failure as an application defect. A failed assertion that Kane blames on the
//! test (wrong steps, wrong expectation) is `NeedsInfo`, never `Reproduced`.

use crate::evidence::Extracted;
use crate::kane::{KaneOutcome, StepOutcome};
use crate::model::{RunSummary, Verdict, VerdictKind};
use crate::render::{LOGIN_HEADING, REPRO_HEADING};

fn verdict(kind: VerdictKind, headline: impl Into<String>) -> Verdict {
    Verdict {
        kind,
        headline: headline.into(),
        stuck_at: None,
        root_cause: None,
        suggested_fix: None,
        severity: None,
        confidence: None,
    }
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or(s).trim().to_string()
}

/// How many real user actions (click, type, ...) the run performed outside the sign-in
/// page. `None` when there is no evidence to count.
fn actions_off_login(ex: Option<&Extracted>, login_path: Option<&str>) -> Option<usize> {
    let ex = ex?;
    if ex.frames.is_empty() {
        return None;
    }
    Some(
        ex.frames
            .iter()
            .filter(|f| {
                matches!(
                    f.kind.as_str(),
                    "click" | "type" | "fill" | "select" | "press"
                )
            })
            .filter(|f| match (login_path, f.url.as_deref()) {
                (Some(lp), Some(u)) => url::Url::parse(u).map(|u| u.path() != lp).unwrap_or(true),
                _ => true,
            })
            .count(),
    )
}

pub fn judge(
    out: &KaneOutcome,
    ex: Option<&Extracted>,
    expects_login: bool,
    login_path: Option<&str>,
) -> Verdict {
    if out.timed_out {
        return verdict(
            VerdictKind::Inconclusive,
            "The browser run hit the time limit before it reached a result.",
        );
    }
    if out.steps.is_empty() {
        let why = out
            .error
            .clone()
            .or_else(|| {
                // Kane prints a skill-update nag on stderr; skip it to find the real message.
                out.stderr_tail
                    .lines()
                    .map(str::trim)
                    .rfind(|l| !l.is_empty() && !l.contains("Skill update available"))
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "Kane produced no result.".into());
        return verdict(
            VerdictKind::Inconclusive,
            format!("Kane did not run: {why}"),
        );
    }

    let find = |h: &str| -> Option<&StepOutcome> { out.steps.iter().find(|s| s.heading == h) };

    if expects_login {
        match find(LOGIN_HEADING) {
            Some(s) if s.status == "passed" => {}
            Some(s) => {
                let why = s
                    .reason
                    .clone()
                    .unwrap_or_else(|| "sign-in did not complete".into());
                return verdict(
                    VerdictKind::Inconclusive,
                    format!(
                        "Could not sign in to the test instance: {}",
                        first_line(&why)
                    ),
                );
            }
            None => return verdict(VerdictKind::Inconclusive, "The sign-in step never started."),
        }
    }

    let Some(repro) = find(REPRO_HEADING) else {
        return verdict(
            VerdictKind::Inconclusive,
            "The reproduction step never started.",
        );
    };

    let tri = ex.map(|e| &e.triage);

    match repro.status.as_str() {
        "passed" => {
            let mut v = verdict(
                VerdictKind::NotReproduced,
                "Kane followed the reported steps and the expected behavior held on this version.",
            );
            v.confidence = None;
            v
        }
        "failed" => {
            let confirmed = repro.bug.as_ref().is_some_and(|b| b.confirmed)
                || repro
                    .reason_code
                    .as_deref()
                    .is_some_and(|c| c.contains("confirmed_product_bug"));
            // A "bug" found before the reporter's steps were even attempted (a missing
            // page, a blocked load) is not a reproduction of the report.
            if confirmed && actions_off_login(ex, login_path) == Some(0) {
                let mut v = verdict(
                    VerdictKind::NeedsInfo,
                    "Kane reported a failure before it performed any of the reported steps, so this is not a reproduction of the report.",
                );
                v.stuck_at = repro.reason.as_deref().map(first_line);
                return v;
            }
            if confirmed {
                let bug = repro.bug.clone().unwrap_or_default();
                let headline = bug
                    .one_liner
                    .clone()
                    .or_else(|| repro.reason.clone())
                    .or_else(|| bug.title.clone())
                    .unwrap_or_else(|| "The reported behavior was observed.".into());
                Verdict {
                    kind: VerdictKind::Reproduced,
                    headline: first_line(&headline),
                    stuck_at: None,
                    root_cause: tri.and_then(|t| t.root_cause.clone()),
                    suggested_fix: tri.and_then(|t| t.suggested_fix.clone()),
                    severity: bug
                        .severity
                        .clone()
                        .or_else(|| tri.and_then(|t| t.severity.clone())),
                    confidence: bug.confidence,
                }
            } else {
                let code = repro.reason_code.clone().unwrap_or_default();
                let is_assert = code.starts_with("assertion_error");
                let reason = repro
                    .reason
                    .clone()
                    .unwrap_or_else(|| "the run failed".into());
                let mut v = if is_assert {
                    verdict(
                        VerdictKind::NeedsInfo,
                        "Kane saw a mismatch at the final check but judged it a problem with the steps or the expectation, not the app. The report may be missing detail.",
                    )
                } else {
                    verdict(
                        VerdictKind::NeedsInfo,
                        "Kane could not complete the reported steps, so the bug was neither confirmed nor ruled out.",
                    )
                };
                v.stuck_at = Some(first_line(&reason));
                v
            }
        }
        other => verdict(
            VerdictKind::Inconclusive,
            format!("The reproduction step ended in an unexpected state: {other}."),
        ),
    }
}

pub fn summarize(
    out: &KaneOutcome,
    ex: Option<&Extracted>,
    share_fallback: Option<String>,
) -> RunSummary {
    let repro = out
        .steps
        .iter()
        .find(|s| s.heading == REPRO_HEADING)
        .or_else(|| out.steps.last());
    let bug = repro.and_then(|s| s.bug.clone());
    let tri = ex.map(|e| e.triage.clone()).unwrap_or_default();
    RunSummary {
        exit_code: out.exit_code,
        timed_out: out.timed_out,
        status: out
            .overall_status
            .clone()
            .unwrap_or_else(|| "unknown".into()),
        reason: repro.and_then(|s| s.reason.clone()),
        reason_code: repro.and_then(|s| s.reason_code.clone()),
        summary: repro.and_then(|s| s.summary.clone()),
        credits: out.total_credits(),
        duration_s: out.duration_s,
        share_url: out.share_url.clone().or(share_fallback),
        failed_step_heading: out
            .steps
            .iter()
            .find(|s| s.status == "failed")
            .map(|s| s.heading.clone()),
        bug_confirmed: bug.as_ref().map(|b| b.confirmed),
        bug_family: bug.as_ref().and_then(|b| b.family.clone()),
        bug_title: bug
            .as_ref()
            .and_then(|b| b.title.clone())
            .or(tri.title.clone()),
        frames: ex.map(|e| e.frames.clone()).unwrap_or_default(),
        strip_file: None,
        signals: ex.map(|e| e.signals.clone()).unwrap_or_default(),
        root_cause: tri.root_cause,
        suggested_fix: tri.suggested_fix,
        severity: tri.severity,
        bug_confidence: bug.and_then(|b| b.confidence).or(tri.confidence),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kane::{BugVerdict, StepOutcome};

    fn step(h: &str, status: &str, code: Option<&str>, bug: Option<BugVerdict>) -> StepOutcome {
        StepOutcome {
            index: 1,
            heading: h.into(),
            status: status.into(),
            reason: Some("because".into()),
            reason_code: code.map(str::to_string),
            bug,
            ..Default::default()
        }
    }

    fn out(steps: Vec<StepOutcome>) -> KaneOutcome {
        KaneOutcome {
            steps,
            saw_stream_start: true,
            ..Default::default()
        }
    }

    fn bug(confirmed: bool) -> BugVerdict {
        BugVerdict {
            confirmed,
            family: Some(
                if confirmed {
                    "application_issue"
                } else {
                    "automation_bug"
                }
                .into(),
            ),
            ..Default::default()
        }
    }

    #[test]
    fn confirmed_bug_is_reproduced() {
        let o = out(vec![step(
            REPRO_HEADING,
            "failed",
            Some("assertion_error.confirmed_product_bug"),
            Some(bug(true)),
        )]);
        assert_eq!(judge(&o, None, false, None).kind, VerdictKind::Reproduced);
    }

    #[test]
    fn failed_assertion_blamed_on_test_is_needs_info() {
        let o = out(vec![step(
            REPRO_HEADING,
            "failed",
            Some("assertion_error"),
            Some(bug(false)),
        )]);
        let v = judge(&o, None, false, None);
        assert_eq!(v.kind, VerdictKind::NeedsInfo);
        assert!(v.stuck_at.is_some());
    }

    #[test]
    fn stuck_before_check_is_needs_info() {
        let o = out(vec![step(
            REPRO_HEADING,
            "failed",
            Some("element_not_found"),
            None,
        )]);
        assert_eq!(judge(&o, None, false, None).kind, VerdictKind::NeedsInfo);
    }

    fn extracted(kinds: &[&str]) -> Extracted {
        let mut ex = Extracted::default();
        for (i, k) in kinds.iter().enumerate() {
            ex.frames.push(crate::model::Frame {
                ordinal: i as u32,
                kind: (*k).into(),
                label: String::new(),
                status: "passed".into(),
                file: String::new(),
                annotated_file: None,
                rect: None,
                url: Some("http://app/".into()),
            });
        }
        ex
    }

    #[test]
    fn a_failure_before_any_user_action_is_not_a_reproduction() {
        let o = out(vec![step(
            REPRO_HEADING,
            "failed",
            Some("assertion_error.confirmed_product_bug"),
            Some(bug(true)),
        )]);
        let only_waiting = extracted(&["navigate", "wait"]);
        assert_eq!(
            judge(&o, Some(&only_waiting), false, None).kind,
            VerdictKind::NeedsInfo
        );
        let acted = extracted(&["navigate", "click", "assert"]);
        assert_eq!(
            judge(&o, Some(&acted), false, None).kind,
            VerdictKind::Reproduced
        );
    }

    #[test]
    fn pass_is_not_reproduced() {
        let o = out(vec![step(REPRO_HEADING, "passed", None, None)]);
        assert_eq!(
            judge(&o, None, false, None).kind,
            VerdictKind::NotReproduced
        );
    }

    #[test]
    fn login_failure_is_inconclusive_not_a_verdict_on_the_bug() {
        let o = out(vec![step(LOGIN_HEADING, "failed", None, None)]);
        assert_eq!(judge(&o, None, true, None).kind, VerdictKind::Inconclusive);
    }

    #[test]
    fn timeout_and_empty_are_inconclusive() {
        let mut o = out(vec![]);
        assert_eq!(judge(&o, None, false, None).kind, VerdictKind::Inconclusive);
        o.timed_out = true;
        assert_eq!(judge(&o, None, false, None).kind, VerdictKind::Inconclusive);
    }

    #[test]
    fn exit_code_is_ignored_in_favor_of_events() {
        // Exit code 1 with a passed repro step must still be NotReproduced.
        let mut o = out(vec![step(REPRO_HEADING, "passed", None, None)]);
        o.exit_code = Some(1);
        assert_eq!(
            judge(&o, None, false, None).kind,
            VerdictKind::NotReproduced
        );
    }
}
