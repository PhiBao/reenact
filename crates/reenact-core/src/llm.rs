//! Minimal OpenAI-compatible chat client. Works against OpenAI, SandBase, or any
//! gateway that speaks `/v1/chat/completions`.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Llm {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    http: reqwest::Client,
}

impl Llm {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            model: model.into(),
            http: reqwest::Client::builder().timeout(Duration::from_secs(180)).build().expect("http client"),
        }
    }

    /// `REENACT_LLM_BASE_URL`, `REENACT_LLM_API_KEY`, `REENACT_LLM_MODEL`.
    pub fn from_env() -> Result<Self> {
        let key = std::env::var("REENACT_LLM_API_KEY").context("REENACT_LLM_API_KEY is not set")?;
        let base = std::env::var("REENACT_LLM_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".into());
        let model = std::env::var("REENACT_LLM_MODEL").unwrap_or_else(|_| "gpt-5.4-mini".into());
        Ok(Self::new(base, key, model))
    }

    pub async fn chat(&self, system: &str, user: &str) -> Result<String> {
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "temperature": 0.1
        });
        let mut last_err = None;
        for attempt in 0..3u32 {
            let resp = self
                .http
                .post(format!("{}/chat/completions", self.base_url))
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    let v: Value = r.json().await.context("decoding LLM response")?;
                    let text = v["choices"][0]["message"]["content"].as_str().unwrap_or_default().to_string();
                    if text.trim().is_empty() {
                        last_err = Some(anyhow::anyhow!("empty LLM response"));
                    } else {
                        return Ok(text);
                    }
                }
                Ok(r) => {
                    let status = r.status();
                    let txt = r.text().await.unwrap_or_default();
                    if status.is_client_error() && status.as_u16() != 429 {
                        bail!("LLM request failed ({status}): {}", txt.chars().take(300).collect::<String>());
                    }
                    last_err = Some(anyhow::anyhow!("LLM {status}"));
                }
                Err(e) => last_err = Some(e.into()),
            }
            tokio::time::sleep(Duration::from_millis(800 * (attempt as u64 + 1))).await;
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("LLM request failed")))
    }
}

/// Pull the first balanced top-level JSON object out of a model reply, tolerating
/// markdown fences and chatter around it.
pub fn extract_json_object(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let start = text.find('{')?;
    let mut depth = 0i32;
    let mut in_str = false;
    let mut esc = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_json_from_fenced_reply() {
        let r = "Sure!\n```json\n{\"a\": \"}\", \"b\": {\"c\": 1}}\n```\nbye";
        assert_eq!(extract_json_object(r), Some("{\"a\": \"}\", \"b\": {\"c\": 1}}"));
    }

    #[test]
    fn none_without_object() {
        assert_eq!(extract_json_object("no json here"), None);
    }
}
