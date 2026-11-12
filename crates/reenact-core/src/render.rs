//! Plan -> Kane `_test.md`. Deterministic; model text only ever appears as
//! sanitized sentences inside fixed scaffolding.

use crate::config::Config;
use crate::model::{Plan, Report};

/// Heading of the step that holds the reproduction. The verdict logic looks
/// for this exact heading to decide whether a failure happened at the check.
pub const REPRO_HEADING: &str = "Reproduce the report";
pub const LOGIN_HEADING: &str = "Sign in";

fn no_md_structure(s: &str) -> String {
    // A step line must not be able to start a new heading, fence or front matter.
    s.trim_start_matches(|c: char| matches!(c, '#' | '`' | '-' | '>' | '@' | '|'))
        .trim()
        .to_string()
}

pub fn render_test_md(cfg: &Config, report: &Report, plan: &Plan, tag: &str) -> String {
    let first_url = match &cfg.login {
        Some(l) => cfg.join(&l.path),
        None => cfg.join(&plan.start_path),
    };
    let title = no_md_structure(
        &report.title.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(100).collect::<String>(),
    );
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("mode: testing\n");
    out.push_str(&format!("url: {first_url}\n"));
    out.push_str("headless: true\n");
    out.push_str(&format!("max_steps: {}\n", cfg.max_steps));
    out.push_str(&format!("tags: [reenact, {tag}]\n"));
    out.push_str("---\n\n");
    out.push_str(&format!("# {title}\n\n"));

    if let Some(login) = &cfg.login {
        out.push_str(&format!("## {LOGIN_HEADING}\n"));
        out.push_str(&format!("{}\n", login.instruction.trim()));
        out.push_str(&format!("Verify {}.\n\n", login.signed_in_check.trim_end_matches('.')));
    }

    out.push_str(&format!("## {REPRO_HEADING}\n"));
    out.push_str(&format!("Open {}.\n", cfg.join(&plan.start_path)));
    out.push_str(&format!(
        "Stay on {} for the whole run and do not open any other website.\n",
        cfg.app_host()
    ));
    if !plan.preconditions.is_empty() {
        out.push_str(&format!(
            "Before the main steps, make sure of this: {}.\n",
            plan.preconditions.iter().map(|s| s.trim_end_matches('.')).collect::<Vec<_>>().join("; ")
        ));
    }
    out.push_str("Do these in order:\n");
    for (i, s) in plan.steps.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", i + 1, no_md_structure(s)));
    }
    let expected = plan.expected.trim_end_matches('.');
    out.push_str(&format!(
        "The reporter says the correct behavior is that {expected}. Verify that {expected}.\n"
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Login;

    fn cfg(login: bool) -> Config {
        Config {
            app_url: "http://localhost:5230".into(),
            description: String::new(),
            allowed_hosts: vec![],
            login: login.then(|| Login {
                path: "/auth".into(),
                instruction: "Sign in with username {{username}} and password {{password}}.".into(),
                signed_in_check: "the memo editor is visible".into(),
                username_env: "U".into(),
                password_env: "P".into(),
            }),
            max_steps: 40,
            timeout_s: 100,
        }
    }

    fn report() -> Report {
        Report { title: "# sneaky\n---".into(), body: String::new(), source_url: None, repo: None, number: Some(7) }
    }

    #[test]
    fn renders_login_then_repro() {
        let plan = Plan {
            in_browser: true,
            start_path: "/".into(),
            steps: vec!["- click Save".into(), "# not a heading".into()],
            expected: "the toast says Saved.".into(),
            ..Default::default()
        };
        let md = render_test_md(&cfg(true), &report(), &plan, "issue-7");
        assert!(md.starts_with("---\nmode: testing\nurl: http://localhost:5230/auth\n"));
        assert!(md.contains("## Sign in\n"));
        assert!(md.contains("## Reproduce the report\n"));
        assert!(md.contains("1. click Save\n"));
        assert!(md.contains("2. not a heading\n"));
        assert!(md.contains("Verify that the toast says Saved.\n"));
        // title cannot smuggle front matter or headings
        assert_eq!(md.matches("\n---\n").count(), 1);
        assert_eq!(md.matches("\n## ").count(), 2);
    }

    #[test]
    fn no_login_starts_on_start_path() {
        let plan = Plan { in_browser: true, start_path: "/settings".into(), steps: vec!["x".into()], expected: "y".into(), ..Default::default() };
        let md = render_test_md(&cfg(false), &report(), &plan, "t");
        assert!(md.contains("url: http://localhost:5230/settings\n"));
        assert!(!md.contains("## Sign in"));
    }
}
