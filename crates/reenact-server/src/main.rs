//! Reenact case-file server. Read-mostly: serves case folders produced by the
//! CLI, accepts bundles from CI behind a bearer token, and serves the web UI
//! with per-case social-card metadata.
//!
//! Storage is the filesystem on purpose: a case is a folder (`case.json`,
//! `strip.png`, `frames/`), which is exactly what the CLI already writes. No
//! database to migrate, and a case can be inspected with `ls`.

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path as AxPath, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reenact_core::model::Case;
use serde::Serialize;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};

const MAX_BUNDLE: usize = 40 * 1024 * 1024;
const MAX_UNPACKED: u64 = 120 * 1024 * 1024;

#[derive(Clone)]
struct AppState {
    cases_dir: Arc<PathBuf>,
    web_dir: Arc<PathBuf>,
    benchmark_file: Arc<PathBuf>,
    upload_token: Option<Arc<String>>,
    public_url: Arc<String>,
}

#[derive(Serialize)]
struct Summary {
    id: String,
    title: String,
    verdict: &'static str,
    state: reenact_core::model::CaseState,
    headline: String,
    created_at: chrono::DateTime<chrono::Utc>,
    has_fix: bool,
    strip: String,
    source_url: Option<String>,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn load_case(dir: &Path, id: &str) -> Option<Case> {
    if !valid_id(id) {
        return None;
    }
    let raw = std::fs::read(dir.join(id).join("case.json")).ok()?;
    serde_json::from_slice(&raw).ok()
}

fn summarize(c: &Case) -> Summary {
    let last = c
        .runs
        .iter()
        .rev()
        .find(|r| r.kind == reenact_core::model::RunKind::Reenact);
    Summary {
        id: c.id.clone(),
        title: c.report.title.clone(),
        verdict: last.map(|r| r.verdict.kind.label()).unwrap_or("Queued"),
        state: c.state,
        headline: last.map(|r| r.verdict.headline.clone()).unwrap_or_default(),
        created_at: c.created_at,
        has_fix: c
            .runs
            .iter()
            .any(|r| r.kind == reenact_core::model::RunKind::VerifyFix),
        strip: last
            .and_then(|r| r.run.strip_file.clone())
            .unwrap_or_else(|| "strip.png".into()),
        source_url: c.report.source_url.clone(),
    }
}

async fn list_cases(State(s): State<AppState>) -> Json<Vec<Summary>> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&*s.cases_dir) {
        for e in rd.flatten() {
            if let Some(id) = e.file_name().to_str() {
                if let Some(c) = load_case(&s.cases_dir, id) {
                    out.push(summarize(&c));
                }
            }
        }
    }
    out.sort_by_key(|s| std::cmp::Reverse(s.created_at));
    Json(out)
}

async fn get_case(State(s): State<AppState>, AxPath(id): AxPath<String>) -> Response {
    match load_case(&s.cases_dir, &id) {
        Some(c) => Json(c).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Serve a file from inside a case folder. Only plain relative paths, only images.
async fn case_file(
    State(s): State<AppState>,
    AxPath((id, rel)): AxPath<(String, String)>,
) -> Response {
    if !valid_id(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let rel = PathBuf::from(&rel);
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let ctype = match rel.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    match std::fs::read(s.cases_dir.join(&id).join(&rel)) {
        Ok(b) => (
            [
                (header::CONTENT_TYPE, ctype),
                (header::CACHE_CONTROL, "public, max-age=300"),
            ],
            b,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn benchmark(State(s): State<AppState>) -> Response {
    match std::fs::read(&*s.benchmark_file) {
        Ok(b) => ([(header::CONTENT_TYPE, "application/json")], b).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "no benchmark results yet").into_response(),
    }
}

/// `PUT /api/benchmark` with the results JSON array (CI or a maintainer, bearer token).
async fn put_benchmark(State(s): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    let Some(token) = &s.upload_token else {
        return (StatusCode::FORBIDDEN, "uploads are disabled").into_response();
    };
    let given = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or("");
    if !constant_time_eq(given.as_bytes(), token.as_bytes()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(v) if v.is_array() => {
            if let Some(p) = s.benchmark_file.parent() {
                let _ = std::fs::create_dir_all(p);
            }
            match std::fs::write(&*s.benchmark_file, &body) {
                Ok(()) => StatusCode::NO_CONTENT.into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        _ => (StatusCode::BAD_REQUEST, "expected a JSON array").into_response(),
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// `POST /api/cases/:id/bundle` with a zip of the case folder.
async fn upload_bundle(
    State(s): State<AppState>,
    AxPath(id): AxPath<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(token) = &s.upload_token else {
        return (StatusCode::FORBIDDEN, "uploads are disabled").into_response();
    };
    let given = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .unwrap_or("");
    if !constant_time_eq(given.as_bytes(), token.as_bytes()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !valid_id(&id) {
        return (StatusCode::BAD_REQUEST, "bad case id").into_response();
    }
    match unpack(&s.cases_dir, &id, &body) {
        Ok(n) => (StatusCode::CREATED, format!("{n} files")).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

fn unpack(base: &Path, id: &str, data: &[u8]) -> Result<usize, String> {
    let mut z = zip::ZipArchive::new(Cursor::new(data)).map_err(|_| "not a zip".to_string())?;
    let tmp = tempfile::tempdir_in(base).map_err(|e| e.to_string())?;
    let mut total = 0u64;
    let mut n = 0;
    let mut saw_case = false;
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        let rel = PathBuf::from(f.name());
        // zip-slip guard + allow-list of file types.
        if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
            return Err("unsafe path in bundle".into());
        }
        let name = rel.to_string_lossy().to_string();
        let ok = name == "case.json"
            || (matches!(
                rel.extension().and_then(|e| e.to_str()),
                Some("png" | "jpg")
            ) && (name.starts_with("frames/") || !name.contains('/')));
        if !ok {
            continue;
        }
        total += f.size();
        if total > MAX_UNPACKED {
            return Err("bundle too large".into());
        }
        let mut buf = Vec::new();
        f.by_ref()
            .take(MAX_UNPACKED)
            .read_to_end(&mut buf)
            .map_err(|e| e.to_string())?;
        if name == "case.json" {
            let c: Case = serde_json::from_slice(&buf)
                .map_err(|_| "case.json is not a valid case".to_string())?;
            if c.id != id {
                return Err("case id mismatch".into());
            }
            saw_case = true;
        }
        let dest = tmp.path().join(&rel);
        if let Some(p) = dest.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        std::fs::write(dest, buf).map_err(|e| e.to_string())?;
        n += 1;
    }
    if !saw_case {
        return Err("bundle has no case.json".into());
    }
    let final_dir = base.join(id);
    let _ = std::fs::remove_dir_all(&final_dir);
    let kept = tmp.keep();
    std::fs::rename(&kept, &final_dir).map_err(|e| e.to_string())?;
    Ok(n)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The SPA shell, with social-card metadata for case pages.
async fn index(State(s): State<AppState>, path: Option<AxPath<String>>) -> Response {
    let Ok(mut html) = std::fs::read_to_string(s.web_dir.join("index.html")) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "web UI is not built. Run `pnpm build` in web/.",
        )
            .into_response();
    };
    if let Some(AxPath(id)) = path {
        if let Some(c) = load_case(&s.cases_dir, &id) {
            let sm = summarize(&c);
            let title = format!("Reenact: {} · {}", sm.verdict, c.report.title);
            let og = format!(
                "<meta property=\"og:title\" content=\"{}\"><meta property=\"og:description\" content=\"{}\"><meta property=\"og:image\" content=\"{}/api/cases/{}/files/strip.png\"><meta name=\"twitter:card\" content=\"summary_large_image\"><title>{}</title>",
                esc(&title), esc(&sm.headline), esc(&s.public_url), esc(&id), esc(&title)
            );
            html = html.replacen("<!--og-->", &og, 1);
        }
    }
    Html(html).into_response()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let cases_dir =
        PathBuf::from(std::env::var("REENACT_CASES_DIR").unwrap_or_else(|_| "data/cases".into()));
    std::fs::create_dir_all(&cases_dir)?;
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let state = AppState {
        cases_dir: Arc::new(cases_dir),
        web_dir: Arc::new(PathBuf::from(
            std::env::var("REENACT_WEB_DIR").unwrap_or_else(|_| "web/dist".into()),
        )),
        benchmark_file: Arc::new(PathBuf::from(
            std::env::var("REENACT_BENCHMARK_FILE")
                .unwrap_or_else(|_| "benchmark/runs/results.json".into()),
        )),
        upload_token: std::env::var("REENACT_UPLOAD_TOKEN")
            .ok()
            .filter(|t| t.len() >= 16)
            .map(Arc::new),
        public_url: Arc::new(
            std::env::var("REENACT_PUBLIC_URL")
                .unwrap_or_else(|_| format!("http://localhost:{port}")),
        ),
    };
    if state.upload_token.is_none() {
        tracing::warn!(
            "REENACT_UPLOAD_TOKEN is unset or shorter than 16 chars: uploads are disabled"
        );
    }

    let api = Router::new()
        .route("/cases", get(list_cases))
        .route("/cases/{id}", get(get_case))
        .route("/cases/{id}/files/{*rel}", get(case_file))
        .route(
            "/cases/{id}/bundle",
            post(upload_bundle).layer(DefaultBodyLimit::max(MAX_BUNDLE)),
        )
        .route(
            "/benchmark",
            get(benchmark)
                .put(put_benchmark)
                .layer(DefaultBodyLimit::max(2 * 1024 * 1024)),
        );

    let spa = Router::new()
        .fallback(get(|s: State<AppState>| index(s, None)))
        .with_state(state.clone());
    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .nest("/api", api)
        .route(
            "/c/{id}",
            get(|s: State<AppState>, p: AxPath<String>| index(s, Some(p))),
        )
        .fallback_service(ServeDir::new(&*state.web_dir).fallback(spa))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(
        "reenact-server on :{port}, cases in {}",
        state.cases_dir.display()
    );
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut buf));
            for (n, b) in files {
                w.start_file(*n, zip::write::SimpleFileOptions::default())
                    .unwrap();
                w.write_all(b).unwrap();
            }
            w.finish().unwrap();
        }
        buf
    }

    #[test]
    fn rejects_zip_slip_and_foreign_files() {
        let base = tempfile::tempdir().unwrap();
        let evil = zip_of(&[("../evil.png", b"x"), ("case.json", b"{}")]);
        assert!(unpack(base.path(), "c-1", &evil).is_err());
        let no_case = zip_of(&[("strip.png", b"x")]);
        assert!(unpack(base.path(), "c-1", &no_case).is_err());
        assert!(valid_id("c-12ab") && !valid_id("../x") && !valid_id(""));
    }

    #[test]
    fn token_compare() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
