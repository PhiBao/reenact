//! GitHub issue comment rendering. Pure function of a `Case`, so it is easy to test.

use crate::model::*;

pub const MARKER_PREFIX: &str = "<!-- reenact:case:";

pub struct Links<'a> {
    /// Base URL of the case folder; image names come from the run records.
    pub strip_url: Option<&'a str>,
    pub compare_url: Option<&'a str>,
    pub case_url: Option<&'a str>,
}

fn quote(s: &str) -> String {
    s.lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn esc_cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

pub fn render_comment(case: &Case, links: &Links) -> String {
    let mut out = String::new();
    out.push_str(&format!("{MARKER_PREFIX}{} -->\n", case.id));

    let last_repro = case.runs.iter().rev().find(|r| r.kind == RunKind::Reenact);
    let last_fix = case
        .runs
        .iter()
        .rev()
        .find(|r| r.kind == RunKind::VerifyFix);
    let confirm = case.runs.iter().rev().find(|r| r.kind == RunKind::Confirm);

    let Some(rec) = last_repro else {
        out.push_str("**Reenact** is reading this report.\n");
        return out;
    };
    let v = &rec.verdict;
    let app = case.app_label.as_deref().unwrap_or("the app");

    out.push_str(&format!("### Reenact: {}\n\n", v.kind.label()));
    if rec.label.contains("retry") {
        if let Some(first) = case
            .runs
            .iter()
            .rev()
            .skip_while(|r| !std::ptr::eq(*r, rec))
            .skip(1)
            .find(|r| r.kind == RunKind::Reenact && r.label == "reenact")
        {
            out.push_str(&format!(
                "_First run said **{}**; this is the result of a second run._\n\n",
                first.verdict.kind.label()
            ));
        }
    }
    out.push_str(&format!("{}\n\n", quote(&v.headline)));

    if let Some(plan) = &case.plan {
        if !plan.expected.is_empty() {
            out.push_str(&format!(
                "**Expected, per the report:** {}\n\n",
                plan.expected
            ));
        }
    }

    if let Some(u) = links.strip_url {
        out.push_str(&format!(
            "![Kane's run, left to right. The last frame is the result.]({u})\n\n"
        ));
    }

    // What Kane did, from the action frames.
    let steps: Vec<&Frame> = rec
        .run
        .frames
        .iter()
        .filter(|f| f.kind != "navigate" && f.kind != "wait" && !f.label.is_empty())
        .collect();
    if !steps.is_empty() {
        out.push_str("| # | What Kane did | Result |\n|---|---|---|\n");
        for (i, f) in steps.iter().enumerate().take(14) {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                i + 1,
                esc_cell(&f.label),
                f.status
            ));
        }
        out.push('\n');
    }

    match v.kind {
        VerdictKind::Reproduced => {
            if let Some(rc) = &v.root_cause {
                out.push_str(&format!("**Kane's read on the cause:** {rc}\n\n"));
            }
            if let Some(sev) = &v.severity {
                out.push_str(&format!("Severity: {sev}. "));
            }
            match confirm {
                Some(c) if c.verdict.kind == VerdictKind::Reproduced => {
                    out.push_str("Confirmed on a second independent run (2 of 2). ");
                }
                Some(c) => {
                    out.push_str(&format!(
                        "A second run said **{}**, so this may be flaky and no fix was attempted. ",
                        c.verdict.kind.label()
                    ));
                }
                None => {}
            }
            out.push_str("Maintainers: comment `/reenact fix` to have an agent attempt a fix that this same run must verify.\n\n");
        }
        VerdictKind::NeedsInfo => {
            if let Some(s) = &v.stuck_at {
                out.push_str(&format!("**Where Kane got stuck:** {s}\n\n"));
            }
            if let Some(plan) = &case.plan {
                if !plan.missing.is_empty() {
                    out.push_str("**To reproduce this, a maintainer would need to know:**\n");
                    for m in &plan.missing {
                        out.push_str(&format!("- {m}\n"));
                    }
                    out.push('\n');
                }
            }
        }
        VerdictKind::NotReproduced => {
            out.push_str("This does not prove the bug is absent: it ran against the version at the app URL, with a fresh data set. Version, data and settings may differ from the reporter's.\n\n");
        }
        VerdictKind::Inconclusive => {
            out.push_str("This says nothing about the bug itself. Reenact could not complete a browser run.\n\n");
        }
    }

    let sig = &rec.run.signals;
    if !sig.console_errors.is_empty() || !sig.failed_requests.is_empty() {
        out.push_str("<details><summary>Browser signals during the run</summary>\n\n");
        for e in &sig.console_errors {
            out.push_str(&format!("- console error: `{}`\n", e.replace('`', "'")));
        }
        for r in &sig.failed_requests {
            out.push_str(&format!("- failed request: `{}`\n", r.replace('`', "'")));
        }
        out.push_str("\n</details>\n\n");
    }

    // Fix section.
    if let Some(fix) = last_fix {
        out.push_str("---\n\n### Fix verification\n\n");
        match fix.verdict.kind {
            VerdictKind::NotReproduced => {
                out.push_str("The same test that reproduced the bug now **passes**.\n\n");
            }
            _ => out.push_str(&format!(
                "After the attempted fix the test result is: **{}**.\n\n",
                fix.verdict.kind.label()
            )),
        }
        if let Some(u) = links.compare_url {
            out.push_str(&format!("![Before (red) and after (green)]({u})\n\n"));
        }
        if let Some(pr) = &case.pr_url {
            out.push_str(&format!("Pull request: {pr}\n\n"));
        }
        if !case.fixes.is_empty() {
            out.push_str("<details><summary>Fix attempts</summary>\n\n");
            for a in &case.fixes {
                let o = a.outcome.map(|o| o.label()).unwrap_or("not verified");
                out.push_str(&format!(
                    "- Attempt {}: {} Result: {}.\n",
                    a.attempt,
                    a.rationale.replace('\n', " "),
                    o
                ));
            }
            out.push_str("\n</details>\n\n");
        }
    }

    if let Some(md) = &case.test_md {
        out.push_str("<details><summary>The test Kane ran (plain English, kept in the repo as a regression test)</summary>\n\n````md\n");
        out.push_str(md);
        out.push_str("\n````\n\n</details>\n\n");
    }

    let mut foot = vec![format!("Ran on {app}")];
    if let Some(u) = rec.run.share_url.as_deref() {
        foot.push(format!("[Kane run]({u})"));
    }
    if let Some(u) = links.case_url {
        foot.push(format!("[Case file]({u})"));
    }
    foot.push(format!(
        "{:.0} credits, {:.0}s",
        rec.run.credits, rec.run.duration_s
    ));
    out.push_str(&format!("<sub>{}</sub>\n", foot.join(" · ")));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::empty_run;

    fn case(kind: VerdictKind) -> Case {
        let mut run = empty_run();
        run.frames.push(Frame {
            ordinal: 2,
            kind: "click".into(),
            label: "click: Apply | coupon".into(),
            status: "passed".into(),
            file: "f.jpg".into(),
            annotated_file: None,
            rect: None,
            url: None,
        });
        Case {
            id: "c-1".into(),
            created_at: chrono::Utc::now(),
            report: Report {
                title: "t".into(),
                body: "b".into(),
                source_url: None,
                repo: None,
                number: Some(1),
            },
            app_url: "http://x".into(),
            app_label: Some("Demo".into()),
            plan: Some(Plan {
                expected: "the total shows $10".into(),
                missing: vec!["Which version?".into()],
                ..Default::default()
            }),
            test_md: Some("# t".into()),
            runs: vec![RunRecord {
                kind: RunKind::Reenact,
                label: "reenact".into(),
                started_at: chrono::Utc::now(),
                verdict: Verdict {
                    kind,
                    headline: "Total went up.".into(),
                    stuck_at: Some("stuck".into()),
                    root_cause: Some("sign flipped".into()),
                    suggested_fix: None,
                    severity: Some("major".into()),
                    confidence: None,
                },
                run,
            }],
            fixes: vec![],
            pr_url: None,
            state: CaseState::Reproduced,
            fix_agent: None,
            fix_diff: None,
            reference_diff: None,
        }
    }

    #[test]
    fn reproduced_comment_has_marker_and_fix_hint() {
        let c = render_comment(
            &case(VerdictKind::Reproduced),
            &Links {
                strip_url: Some("https://x/s.png"),
                compare_url: None,
                case_url: None,
            },
        );
        assert!(c.starts_with("<!-- reenact:case:c-1 -->"));
        assert!(c.contains("Reenact: Reproduced"));
        assert!(c.contains("/reenact fix"));
        assert!(c.contains("![Kane's run"));
        assert!(c.contains("Apply \\| coupon"));
    }

    #[test]
    fn needs_info_lists_questions() {
        let c = render_comment(
            &case(VerdictKind::NeedsInfo),
            &Links {
                strip_url: None,
                compare_url: None,
                case_url: None,
            },
        );
        assert!(c.contains("Which version?"));
        assert!(c.contains("Where Kane got stuck"));
        assert!(!c.contains("/reenact fix"));
    }
}
