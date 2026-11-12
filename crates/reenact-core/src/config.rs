//! `.reenact.yml`: what a maintainer tells Reenact about their app. Deliberately small.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Where the app under test is running, e.g. `http://localhost:5230`.
    pub app_url: String,
    /// One sentence about the app. Helps the compiler read the report.
    #[serde(default)]
    pub description: String,
    /// Extra hosts (besides `app_url`) a plan may mention. Usually empty.
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    #[serde(default)]
    pub login: Option<Login>,
    /// Max Kane steps per run.
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
    /// Wall clock cap for one Kane run, seconds.
    #[serde(default = "default_timeout")]
    pub timeout_s: u64,
}

fn default_max_steps() -> u32 {
    40
}
fn default_timeout() -> u64 {
    420
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login {
    /// Path of the sign-in page.
    pub path: String,
    /// Plain-English sign-in step. May use `{{username}}` and `{{password}}`.
    pub instruction: String,
    /// What the page shows once signed in. Used as the check at the end of the sign-in step.
    pub signed_in_check: String,
    /// Environment variable names holding the test credentials.
    #[serde(default = "default_user_env")]
    pub username_env: String,
    #[serde(default = "default_pass_env")]
    pub password_env: String,
}

fn default_user_env() -> String {
    "REENACT_USERNAME".into()
}
fn default_pass_env() -> String {
    "REENACT_PASSWORD".into()
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let cfg: Config =
            serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        let u = url::Url::parse(&self.app_url).context("app_url is not a valid URL")?;
        anyhow::ensure!(
            matches!(u.scheme(), "http" | "https"),
            "app_url must be http(s)"
        );
        Ok(())
    }

    /// `host` and `host:port` forms of every host a plan may mention.
    pub fn all_hosts(&self) -> Vec<String> {
        let mut v = self.allowed_hosts.clone();
        if let Ok(u) = url::Url::parse(&self.app_url) {
            if let Some(h) = u.host_str() {
                v.push(h.to_string());
                if let Some(p) = u.port() {
                    v.push(format!("{h}:{p}"));
                }
            }
        }
        v
    }

    pub fn app_host(&self) -> String {
        url::Url::parse(&self.app_url)
            .ok()
            .and_then(|u| {
                u.host_str().map(|h| match u.port() {
                    Some(p) => format!("{h}:{p}"),
                    None => h.to_string(),
                })
            })
            .unwrap_or_default()
    }

    pub fn join(&self, path: &str) -> String {
        format!("{}{}", self.app_url.trim_end_matches('/'), path)
    }
}
