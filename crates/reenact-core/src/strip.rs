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
    Ok(image::open(dir.join(file))
        .with_context(|| format!("opening {file}"))?
        .to_rgba8())
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
        .find(|f| {
            matches!(
                f.kind.as_str(),
                "click" | "type" | "fill" | "select" | "press"
            ) && f.rect.is_some()
        })
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

    // One crop window for every panel, so the panels line up and the page is legible.
    let trig = trigger_rect(frames);
    let mut thumbs: Vec<RgbaImage> = Vec::new();
    let mut ring: Option<[f64; 4]> = None; // trigger rect in the last thumbnail's pixels
    for (i, f) in picked.iter().enumerate() {
        let mut img = load(dir, &f.file)?;
        let mut crop_origin = (0.0, 0.0);
        let sx = img.width() as f64 / viewport.0 as f64;
        let sy = img.height() as f64 / viewport.1 as f64;
        if let Some(r) = trig {
            let (cx, cy, cw, ch) =
                crop_window(img.width(), img.height(), r[0] * sx, r[1] * sy, r[2] * sx);
            img = image::imageops::crop_imm(&img, cx, cy, cw, ch).to_image();
            crop_origin = (cx as f64, cy as f64);
        }
        let th = (img.height() as f64 * thumb_w as f64 / img.width() as f64).round() as u32;
        let scale = thumb_w as f64 / img.width() as f64;
        if i == picked.len() - 1 {
            if let Some(r) = trig {
                ring = Some([
                    (r[0] * sx - crop_origin.0) * scale,
                    (r[1] * sy - crop_origin.1) * scale,
                    r[2] * sx * scale,
                    r[3] * sy * scale,
                ]);
            }
        }
        thumbs.push(image::imageops::resize(
            &img,
            thumb_w,
            th.max(1),
            FilterType::Triangle,
        ));
    }
    let max_h = thumbs.iter().map(|t| t.height()).max().unwrap_or(1);
    let total_w = margin * 2 + thumbs.len() as u32 * thumb_w + (thumbs.len() as u32 - 1) * gutter;
    let total_h = margin * 2 + max_h;
    let mut canvas = RgbaImage::from_pixel(total_w, total_h, PAPER);

    let last = thumbs.len() - 1;
    for (i, t) in thumbs.iter_mut().enumerate() {
        if i == last {
            if let Some(r) = ring {
                let pad = 6.0;
                border(
                    t,
                    (r[0] - pad).round() as i64,
                    (r[1] - pad).round() as i64,
                    (r[2] + pad * 2.0).round().max(8.0) as i64,
                    (r[3] + pad * 2.0).round().max(8.0) as i64,
                    3,
                    verdict_color(kind),
                );
            }
        }
        let x0 = margin + i as u32 * (thumb_w + gutter);
        image::imageops::overlay(&mut canvas, t, x0 as i64, margin as i64);
        // Panel outline: thin ink; the final panel is thick in the verdict color.
        if i == last {
            border(
                &mut canvas,
                x0 as i64 - 4,
                margin as i64 - 4,
                thumb_w as i64 + 8,
                t.height() as i64 + 8,
                4,
                verdict_color(kind),
            );
        } else {
            border(
                &mut canvas,
                x0 as i64 - 1,
                margin as i64 - 1,
                thumb_w as i64 + 2,
                t.height() as i64 + 2,
                1,
                INK,
            );
        }
    }
    canvas
        .save(out)
        .with_context(|| format!("writing {}", out.display()))?;
    Ok(())
}

/// Crop window (x, y, w, h) in image pixels around an element, favouring the
/// area below it where totals and messages tend to appear.
fn crop_window(img_w: u32, img_h: u32, rx: f64, ry: f64, rw: f64) -> (u32, u32, u32, u32) {
    let (iw, ih) = (img_w as f64, img_h as f64);
    let cw = (iw * 0.5).max(720.0).min(iw);
    let ch = (ih * 0.46).max(420.0).min(ih);
    let cx = (rx + rw / 2.0 - cw / 2.0).clamp(0.0, iw - cw);
    let cy = (ry - ch * 0.22).clamp(0.0, ih - ch);
    (cx as u32, cy as u32, cw as u32, ch as u32)
}

/// Load a frame, crop to the neighbourhood of `rect` (where the user acted and
/// where the result shows), and ring the element. Without a rect the whole
/// frame is returned.
fn framed(
    dir: &Path,
    frame: &Frame,
    rect: Option<[f64; 4]>,
    color: Rgba<u8>,
    viewport: (u32, u32),
) -> Result<RgbaImage> {
    let img = load(dir, &frame.file)?;
    let Some(r) = rect else { return Ok(img) };
    let sx = img.width() as f64 / viewport.0 as f64;
    let sy = img.height() as f64 / viewport.1 as f64;
    let (rx, ry, rw, rh) = (r[0] * sx, r[1] * sy, r[2] * sx, r[3] * sy);
    // A window that favours the area below the element, where totals and messages appear.
    let (cx, cy, cw, ch) = crop_window(img.width(), img.height(), rx, ry, rw);
    let mut crop = image::imageops::crop_imm(&img, cx, cy, cw, ch).to_image();
    border(
        &mut crop,
        (rx - cx as f64 - 8.0) as i64,
        (ry - cy as f64 - 8.0) as i64,
        (rw + 16.0) as i64,
        (rh + 16.0) as i64,
        4,
        color,
    );
    Ok(crop)
}

/// One cropped frame with a ring around `rect`, for the before/after slider.
pub fn compose_single(
    dir: &Path,
    frame: &Frame,
    rect: Option<[f64; 4]>,
    color: Rgba<u8>,
    viewport: (u32, u32),
    out: &Path,
) -> Result<()> {
    framed(dir, frame, rect, color, viewport)?
        .save(out)
        .with_context(|| format!("writing {}", out.display()))?;
    Ok(())
}

pub fn red() -> Rgba<u8> {
    RED
}
pub fn green() -> Rgba<u8> {
    GREEN
}

/// Before/after panel: two cropped frames side by side, red vs green.
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
    let scale = |img: RgbaImage| {
        let th = (img.height() as f64 * thumb_w as f64 / img.width() as f64).round() as u32;
        image::imageops::resize(&img, thumb_w, th.max(1), FilterType::Triangle)
    };
    let a = scale(framed(before_dir, before, before_rect, RED, viewport)?);
    let b = scale(framed(after_dir, after, before_rect, GREEN, viewport)?);
    let h = a.height().max(b.height());
    let mut canvas =
        RgbaImage::from_pixel(margin * 2 + thumb_w * 2 + gutter, margin * 2 + h, PAPER);
    image::imageops::overlay(&mut canvas, &a, margin as i64, margin as i64);
    image::imageops::overlay(
        &mut canvas,
        &b,
        (margin + thumb_w + gutter) as i64,
        margin as i64,
    );
    border(
        &mut canvas,
        margin as i64 - 4,
        margin as i64 - 4,
        thumb_w as i64 + 8,
        a.height() as i64 + 8,
        4,
        RED,
    );
    border(
        &mut canvas,
        (margin + thumb_w + gutter) as i64 - 4,
        margin as i64 - 4,
        thumb_w as i64 + 8,
        b.height() as i64 + 8,
        4,
        GREEN,
    );
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
        compose_strip(
            dir.path(),
            &ex.frames,
            VerdictKind::Reproduced,
            ex.viewport.unwrap(),
            &out,
        )
        .unwrap();
        let img = image::open(&out).unwrap();
        assert!(img.width() > 1500 && img.height() > 200);
        assert!(trigger_rect(&ex.frames).is_some());
    }

    #[test]
    fn compare_crops_to_the_evidence() {
        let pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/coupon-bug.evidence");
        let dir = tempfile::tempdir().unwrap();
        let ex = evidence::extract(&pack, dir.path(), "").unwrap();
        let last = ex.frames.last().unwrap();
        let rect = trigger_rect(&ex.frames);
        let out = std::env::temp_dir().join("reenact-compare-preview.png");
        compose_compare(dir.path(), last, rect, dir.path(), last, (1920, 1080), &out).unwrap();
        let img = image::open(&out).unwrap();
        // two cropped panels, much shorter than two full 16:9 frames
        assert!(img.height() < 600, "height {}", img.height());
    }

    #[test]
    fn pick_keeps_first_and_last() {
        let mk = |o| Frame {
            ordinal: o,
            kind: "click".into(),
            label: String::new(),
            status: String::new(),
            file: String::new(),
            annotated_file: None,
            rect: None,
            url: None,
        };
        let frames: Vec<Frame> = (1..=12).map(mk).collect();
        let p = pick_frames(&frames, 5);
        assert_eq!(p.first().unwrap().ordinal, 1);
        assert_eq!(p.last().unwrap().ordinal, 12);
        assert_eq!(p.len(), 5);
    }
}
