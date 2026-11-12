//! Composes the evidence strip (and the before/after comparison) from Kane's
//! screenshots. No text rendering on purpose: captions live in markdown/HTML
//! where they stay selectable and translatable.

use crate::model::{Frame, VerdictKind};
use anyhow::{Context, Result};
use image::{imageops::FilterType, Rgba, RgbaImage};
use std::path::Path;

const PAPER: Rgba<u8> = Rgba([0xF4, 0xF1, 0xEA, 0xFF]);
const INK: Rgba<u8> = Rgba([0x14, 0x14, 0x14, 0xFF]);
const RED: Rgba<u8> = Rgba([0xD7, 0x30, 0x1F, 0xFF]);
const GREEN: Rgba<u8> = Rgba([0x1F, 0x7A, 0x4D, 0xFF]);
const AMBER: Rgba<u8> = Rgba([0xB7, 0x79, 0x1F, 0xFF]);

pub fn verdict_color(kind: VerdictKind) -> Rgba<u8> {
    match kind {
        VerdictKind::Reproduced => RED,
        VerdictKind::NotReproduced => GREEN,
        VerdictKind::NeedsInfo => AMBER,
        VerdictKind::Inconclusive => INK,
    }
}

fn border(img: &mut RgbaImage, x: i64, y: i64, w: i64, h: i64, t: i64, c: Rgba<u8>) {
    let (iw, ih) = (img.width() as i64, img.height() as i64);
    for yy in y..y + h {
        for xx in x..x + w {
            let on_edge = xx < x + t || xx >= x + w - t || yy < y + t || yy >= y + h - t;
            if on_edge && xx >= 0 && yy >= 0 && xx < iw && yy < ih {
                img.put_pixel(xx as u32, yy as u32, c);
            }
        }
    }
}

fn load(dir: &Path, file: &str) -> Result<RgbaImage> {
    Ok(image::open(dir.join(file)).with_context(|| format!("opening {file}"))?.to_rgba8())
}

/// Pick up to `n` frames: always the first and the last, the rest spread evenly.
pub fn pick_frames(frames: &[Frame], n: usize) -> Vec<&Frame> {
    if frames.len() <= n {
        return frames.iter().collect();
    }
    let mut idx: Vec<usize> = (0..n).map(|i| i * (frames.len() - 1) / (n - 1)).collect();
    idx.dedup();
    idx.into_iter().map(|i| &frames[i]).collect()
}

/// The element the last user action touched, in viewport pixels. This is the
/// "trigger" we circle on the final frame.
pub fn trigger_rect(frames: &[Frame]) -> Option<[f64; 4]> {
    frames
        .iter()
        .rev()
        .find(|f| matches!(f.kind.as_str(), "click" | "type" | "fill" | "select" | "press") && f.rect.is_some())
        .and_then(|f| f.rect)
}

/// Build the horizontal strip. The last panel gets a verdict-colored frame and,
/// when `rect` is known, a ring around the element that triggered the behavior.
pub fn compose_strip(
    dir: &Path,
    frames: &[Frame],
    kind: VerdictKind,
    viewport: (u32, u32),
    out: &Path,
) -> Result<()> {
    anyhow::ensure!(!frames.is_empty(), "no frames to compose");
    let picked = pick_frames(frames, 5);
    let thumb_w: u32 = 520;
    let gutter: u32 = 14;
    let margin: u32 = 24;

    let mut thumbs: Vec<RgbaImage> = Vec::new();
    for f in &picked {
        let img = load(dir, &f.file)?;
        let th = (img.height() as f64 * thumb_w as f64 / img.width() as f64).round() as u32;
        thumbs.push(image::imageops::resize(&img, thumb_w, th.max(1), FilterType::Triangle));
    }
    let max_h = thumbs.iter().map(|t| t.height()).max().unwrap_or(1);
    let total_w = margin * 2 + thumbs.len() as u32 * thumb_w + (thumbs.len() as u32 - 1) * gutter;
    let total_h = margin * 2 + max_h;
    let mut canvas = RgbaImage::from_pixel(total_w, total_h, PAPER);

    let last = thumbs.len() - 1;
    for (i, t) in thumbs.iter_mut().enumerate() {
        if i == last {
            if let Some(r) = trigger_rect(frames) {
                let sx = t.width() as f64 / viewport.0 as f64;
                let sy = t.height() as f64 / viewport.1 as f64;
                let pad = 6.0;
                let (x, y) = (((r[0]) * sx - pad).round() as i64, ((r[1]) * sy - pad).round() as i64);
                let (w, h) = (((r[2]) * sx + pad * 2.0).round() as i64, ((r[3]) * sy + pad * 2.0).round() as i64);
                border(t, x, y, w.max(8), h.max(8), 3, verdict_color(kind));
            }
        }
        let x0 = margin + i as u32 * (thumb_w + gutter);
        image::imageops::overlay(&mut canvas, t, x0 as i64, margin as i64);
        // Panel outline: thin ink; the final panel is thick in the verdict color.
        if i == last {
            border(&mut canvas, x0 as i64 - 4, margin as i64 - 4, thumb_w as i64 + 8, t.height() as i64 + 8, 4, verdict_color(kind));
        } else {
            border(&mut canvas, x0 as i64 - 1, margin as i64 - 1, thumb_w as i64 + 2, t.height() as i64 + 2, 1, INK);
        }
    }
    canvas.save(out).with_context(|| format!("writing {}", out.display()))?;
    Ok(())
}

/// One full-size frame with an optional ring around `rect`, for the before/after slider.
pub fn compose_single(dir: &Path, frame: &Frame, rect: Option<[f64; 4]>, color: Rgba<u8>, viewport: (u32, u32), out: &Path) -> Result<()> {
    let mut img = load(dir, &frame.file)?;
    if let Some(r) = rect {
        let sx = img.width() as f64 / viewport.0 as f64;
        let sy = img.height() as f64 / viewport.1 as f64;
        border(&mut img, (r[0] * sx - 8.0) as i64, (r[1] * sy - 8.0) as i64, (r[2] * sx + 16.0) as i64, (r[3] * sy + 16.0) as i64, 4, color);
    }
    img.save(out).with_context(|| format!("writing {}", out.display()))?;
    Ok(())
}

pub fn red() -> Rgba<u8> {
    RED
}
pub fn green() -> Rgba<u8> {
    GREEN
}

/// Before/after panel: two final frames side by side, red vs green.
pub fn compose_compare(
    before_dir: &Path,
    before: &Frame,
    before_rect: Option<[f64; 4]>,
    after_dir: &Path,
    after: &Frame,
    viewport: (u32, u32),
    out: &Path,
) -> Result<()> {
    let thumb_w: u32 = 760;
    let gutter: u32 = 24;
    let margin: u32 = 28;
    let mk = |dir: &Path, f: &Frame, rect: Option<[f64; 4]>, c: Rgba<u8>| -> Result<RgbaImage> {
        let img = load(dir, &f.file)?;
        let th = (img.height() as f64 * thumb_w as f64 / img.width() as f64).round() as u32;
        let mut t = image::imageops::resize(&img, thumb_w, th.max(1), FilterType::Triangle);
        if let Some(r) = rect {
            let sx = t.width() as f64 / viewport.0 as f64;
            let sy = t.height() as f64 / viewport.1 as f64;
            border(&mut t, (r[0] * sx - 6.0) as i64, (r[1] * sy - 6.0) as i64, (r[2] * sx + 12.0) as i64, (r[3] * sy + 12.0) as i64, 3, c);
        }
        Ok(t)
    };
    let a = mk(before_dir, before, before_rect, RED)?;
    let b = mk(after_dir, after, before_rect, GREEN)?;
    let h = a.height().max(b.height());
    let mut canvas = RgbaImage::from_pixel(margin * 2 + thumb_w * 2 + gutter, margin * 2 + h, PAPER);
    image::imageops::overlay(&mut canvas, &a, margin as i64, margin as i64);
    image::imageops::overlay(&mut canvas, &b, (margin + thumb_w + gutter) as i64, margin as i64);
    border(&mut canvas, margin as i64 - 4, margin as i64 - 4, thumb_w as i64 + 8, a.height() as i64 + 8, 4, RED);
    border(&mut canvas, (margin + thumb_w + gutter) as i64 - 4, margin as i64 - 4, thumb_w as i64 + 8, b.height() as i64 + 8, 4, GREEN);
    canvas.save(out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence;

    #[test]
    fn composes_strip_from_real_pack() {
        let pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/coupon-bug.evidence");
        let dir = tempfile::tempdir().unwrap();
        let ex = evidence::extract(&pack, dir.path(), "").unwrap();
        let out = dir.path().join("strip.png");
        compose_strip(dir.path(), &ex.frames, VerdictKind::Reproduced, ex.viewport.unwrap(), &out).unwrap();
        let img = image::open(&out).unwrap();
        assert!(img.width() > 1500 && img.height() > 200);
        assert!(trigger_rect(&ex.frames).is_some());
    }

    #[test]
    fn pick_keeps_first_and_last() {
        let mk = |o| Frame { ordinal: o, kind: "click".into(), label: String::new(), status: String::new(), file: String::new(), annotated_file: None, rect: None, url: None };
        let frames: Vec<Frame> = (1..=12).map(mk).collect();
        let p = pick_frames(&frames, 5);
        assert_eq!(p.first().unwrap().ordinal, 1);
        assert_eq!(p.last().unwrap().ordinal, 12);
        assert_eq!(p.len(), 5);
    }
}
