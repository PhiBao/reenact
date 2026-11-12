//! Reads a sealed Kane `.evidence` pack (a zip) and pulls out the pieces Reenact
//! shows: per-step frames, Kane's failure triage, console errors, failed requests.

use crate::model::{Frame, Signals};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct Triage {
    pub title: Option<String>,
    pub root_cause: Option<String>,
    pub suggested_fix: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub confidence: Option<f64>,
    pub error_message: Option<String>,
    pub failed_step_ordinal: Option<u32>,
}

#[derive(Debug, Default)]
pub struct Extracted {
    pub frames: Vec<Frame>,
    pub triage: Triage,
    pub signals: Signals,
    /// Viewport width/height reported by the run (for scaling element rects).
    pub viewport: Option<(u32, u32)>,
}

#[derive(Deserialize)]
struct StepJson {
    kind: Option<String>,
    status: Option<String>,
    summary: Option<String>,
    url: Option<String>,
    element_rect: Option<Rect>,
    ordinal: Option<u32>,
}

#[derive(Deserialize)]
struct Rect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn read_entry(z: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Option<Vec<u8>> {
    let mut f = z.by_name(name).ok()?;
    let mut buf = Vec::with_capacity(f.size() as usize);
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// Extract everything useful from `pack` and write frame images into `out_dir`
/// as `frame-<ordinal>.<ext>` and `frame-<ordinal>-annotated.png`.
pub fn extract(pack: &Path, out_dir: &Path, name_prefix: &str) -> Result<Extracted> {
    let file = std::fs::File::open(pack).with_context(|| format!("opening {}", pack.display()))?;
    let mut z = zip::ZipArchive::new(file).context("evidence pack is not a valid zip")?;
    std::fs::create_dir_all(out_dir)?;

    // Find the (first) test directory: tests/<id>/
    let names: Vec<String> = z.file_names().map(|s| s.to_string()).collect();
    let test_dir = names
        .iter()
        .find_map(|n| n.strip_prefix("tests/").and_then(|r| r.split('/').next()).map(|id| format!("tests/{id}")))
        .context("no tests/ directory in evidence pack")?;

    let mut out = Extracted::default();

    // result.yaml: viewport.
    if let Some(b) = read_entry(&mut z, &format!("{test_dir}/result.yaml")) {
        if let Ok(v) = serde_yaml::from_slice::<Value>(&b) {
            if let Some(res) = v["environment"]["resolution"].as_str() {
                if let Some((w, h)) = res.split_once('x') {
                    if let (Ok(w), Ok(h)) = (w.parse(), h.parse()) {
                        out.viewport = Some((w, h));
                    }
                }
            }
        }
    }

    // Steps.
    let step_prefix = format!("{test_dir}/steps/");
    let mut step_dirs: Vec<String> = names
        .iter()
        .filter_map(|n| n.strip_prefix(&step_prefix))
        .filter_map(|r| r.split('/').next())
        .map(|s| s.to_string())
        .collect();
    step_dirs.sort();
    step_dirs.dedup();

    for dir in step_dirs {
        let base = format!("{step_prefix}{dir}");
        let Some(sj) = read_entry(&mut z, &format!("{base}/step.json")) else { continue };
        let Ok(step): Result<StepJson, _> = serde_json::from_slice(&sj) else { continue };
        let ordinal = step.ordinal.or_else(|| dir.split('-').next().and_then(|s| s.parse().ok())).unwrap_or(0);

        let (shot_name, shot) = if let Some(b) = read_entry(&mut z, &format!("{base}/screenshot.jpg")) {
            ("jpg", Some(b))
        } else if let Some(b) = read_entry(&mut z, &format!("{base}/screenshot.png")) {
            ("png", Some(b))
        } else {
            ("", None)
        };
        let Some(shot) = shot else { continue };
        let file = format!("{name_prefix}frame-{ordinal:02}.{shot_name}");
        std::fs::write(out_dir.join(&file), shot)?;

        let annotated_file = if let Some(b) = read_entry(&mut z, &format!("{base}/annotated.png")) {
            let f = format!("{name_prefix}frame-{ordinal:02}-annotated.png");
            std::fs::write(out_dir.join(&f), b)?;
            Some(f)
        } else {
            None
        };

        let status = step.status.clone().unwrap_or_default();
        if status == "failed" && out.triage.failed_step_ordinal.is_none() {
            out.triage.failed_step_ordinal = Some(ordinal);
        }
        out.frames.push(Frame {
            ordinal,
            kind: step.kind.clone().unwrap_or_default(),
            label: step.summary.clone().unwrap_or_default(),
            status,
            file,
            annotated_file,
            rect: step.element_rect.map(|r| [r.x, r.y, r.width, r.height]),
            url: step.url.clone(),
        });

        // Triage lives next to the failing step.
        if let Some(fb) = read_entry(&mut z, &format!("{base}/failure.yaml")) {
            if let Ok(v) = serde_yaml::from_slice::<Value>(&fb) {
                let s = |p: &Value| p.as_str().map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "));
                out.triage.error_message = out.triage.error_message.take().or_else(|| s(&v["error"]["message"]));
                out.triage.title = out.triage.title.take().or_else(|| s(&v["title"]));
                out.triage.root_cause = out.triage.root_cause.take().or_else(|| s(&v["triage"]["rca"]["root_cause"]));
                out.triage.category = out.triage.category.take().or_else(|| s(&v["triage"]["rca"]["category"]));
                out.triage.confidence = out.triage.confidence.or_else(|| v["triage"]["rca"]["confidence"].as_f64());
                out.triage.severity = out.triage.severity.take().or_else(|| s(&v["triage"]["severity"]));
                out.triage.suggested_fix =
                    out.triage.suggested_fix.take().or_else(|| s(&v["triage"]["suggested_fix"]["summary"]));
            }
        }
    }
    out.frames.sort_by_key(|f| f.ordinal);

    // Console errors.
    if let Some(b) = read_entry(&mut z, &format!("{test_dir}/logs/0-console.ndjson")) {
        for line in String::from_utf8_lossy(&b).lines() {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if v["level"].as_str() == Some("error") {
                    let text = v["text"].as_str().unwrap_or_default();
                    let url = v["url"].as_str().unwrap_or_default();
                    // Favicon 404s are noise on dev servers.
                    if url.ends_with("/favicon.ico") {
                        continue;
                    }
                    let msg: String = text.chars().take(200).collect();
                    if !msg.is_empty() && out.signals.console_errors.len() < 8 {
                        out.signals.console_errors.push(msg);
                    }
                }
            }
        }
    }

    // Failed requests from the HAR.
    if let Some(b) = read_entry(&mut z, &format!("{test_dir}/logs/0-network.har")) {
        if let Ok(v) = serde_json::from_slice::<Value>(&b) {
            if let Some(entries) = v["log"]["entries"].as_array() {
                for e in entries {
                    let status = e["response"]["status"].as_u64().unwrap_or(0);
                    let url = e["request"]["url"].as_str().unwrap_or_default();
                    if status >= 400 && !url.ends_with("/favicon.ico") && out.signals.failed_requests.len() < 8 {
                        let method = e["request"]["method"].as_str().unwrap_or("GET");
                        let short: String = url.chars().take(120).collect();
                        out.signals.failed_requests.push(format!("{status} {method} {short}"));
                    }
                }
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_kane_pack() {
        let pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/coupon-bug.evidence");
        let dir = tempfile::tempdir().unwrap();
        let ex = extract(&pack, dir.path(), "").unwrap();
        assert!(ex.frames.len() >= 4, "frames: {}", ex.frames.len());
        assert_eq!(ex.viewport, Some((1920, 1080)));
        assert!(ex.triage.root_cause.as_deref().unwrap().contains("wrong direction"));
        assert_eq!(ex.triage.severity.as_deref(), Some("major"));
        assert!(ex.triage.suggested_fix.is_some());
        // The click on the coupon button carries an element rect.
        assert!(ex.frames.iter().any(|f| f.kind == "click" && f.rect.is_some()));
        for f in &ex.frames {
            assert!(dir.path().join(&f.file).exists());
        }
    }
}
