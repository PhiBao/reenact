//! Composes the evidence strip (and the before/after comparison) from Kane's
//! screenshots. No text rendering on purpose: captions live in markdown/HTML
//! where they stay selectable and translatable.
//!
//! Screenshots are full pages and most of a page is irrelevant to a bug. Every
//! image here is cropped to the part that matters: the element the user acted on,
//! plus whatever on the page changed between the frames being compared.

use crate::model::{Frame, VerdictKind};
use anyhow::{Context, Result};
use image::{imageops::FilterType, Rgba, RgbaImage};
use std::path::Path;

const PAPER: Rgba<u8> = Rgba([0xF4, 0xF1, 0xEA, 0xFF]);
const INK: Rgba<u8> = Rgba([0x14, 0x14, 0x14, 0xFF]);
const RED: Rgba<u8> = Rgba([0xD7, 0x30, 0x1F, 0xFF]);
const GREEN: Rgba<u8> = Rgba([0x1F, 0x7A, 0x4D, 0xFF]);
const AMBER: Rgba<u8> = Rgba([0xB7, 0x79, 0x1F, 0xFF]);

/// A crop window in image pixels: x, y, width, height.
pub type Window = (u32, u32, u32, u32);
type BBox = (f64, f64, f64, f64);

pub fn verdict_color(kind: VerdictKind) -> Rgba<u8> {
    match kind {
        VerdictKind::Reproduced => RED,
        VerdictKind::NotReproduced => GREEN,
        VerdictKind::NeedsInfo => AMBER,
        VerdictKind::Inconclusive => INK,
    }
}

pub fn red() -> Rgba<u8> {
    RED
}

pub fn green() -> Rgba<u8> {
    GREEN
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

fn lum(p: &Rgba<u8>) -> i32 {
    (p[0] as i32 * 30 + p[1] as i32 * 59 + p[2] as i32 * 11) / 100
}

/// Bounding box of the pixels that differ between two screenshots, in the pixels
/// of `a`. `None` when (almost) nothing changed.
fn diff_bbox(a: &RgbaImage, b: &RgbaImage) -> Option<BBox> {
    let b = if b.dimensions() == a.dimensions() {
        std::borrow::Cow::Borrowed(b)
    } else {
        std::borrow::Cow::Owned(image::imageops::resize(
            b,
            a.width(),
            a.height(),
            FilterType::Triangle,
        ))
    };
    let step = 3usize;
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let mut n = 0u32;
    for y in (0..a.height()).step_by(step) {
        for x in (0..a.width()).step_by(step) {
            if (lum(a.get_pixel(x, y)) - lum(b.get_pixel(x, y))).abs() > 40 {
                n += 1;
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    (n >= 12).then(|| (x0 as f64, y0 as f64, (x1 - x0) as f64, (y1 - y0) as f64))
}

/// Grow the union of `boxes` into a comfortable window: padded, never tiny, roughly
/// landscape, always inside the image.
fn window_around(boxes: &[BBox], iw: u32, ih: u32) -> Option<Window> {
    let first = boxes.first()?;
    let (mut x0, mut y0, mut x1, mut y1) = (first.0, first.1, first.0 + first.2, first.1 + first.3);
    for b in &boxes[1..] {
        x0 = x0.min(b.0);
        y0 = y0.min(b.1);
        x1 = x1.max(b.0 + b.2);
        y1 = y1.max(b.1 + b.3);
    }
    let pad = 56.0;
    let (iwf, ihf) = (iw as f64, ih as f64);
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let mut w = (x1 - x0 + 2.0 * pad).max(iwf * 0.34).max(640.0);
    let mut h = (y1 - y0 + 2.0 * pad).max(ihf * 0.34).max(360.0);
    if w < h * 1.5 {
        w = h * 1.5;
    }
    if h < w / 2.4 {
        h = w / 2.4;
    }
    let (w, h) = (w.min(iwf), h.min(ihf));
    let x = (cx - w / 2.0).clamp(0.0, iwf - w);
    let y = (cy - h / 2.0).clamp(0.0, ihf - h);
    Some((x as u32, y as u32, w as u32, h as u32))
}

fn scaled_rect(r: [f64; 4], img: &RgbaImage, viewport: (u32, u32)) -> BBox {
    let sx = img.width() as f64 / viewport.0 as f64;
    let sy = img.height() as f64 / viewport.1 as f64;
    (r[0] * sx, r[1] * sy, r[2] * sx, r[3] * sy)
}

/// The part of the page worth showing for a before/after pair: where the user acted
/// and where the page changed between the two frames.
pub fn change_window(
    before_dir: &Path,
    before: &Frame,
    after_dir: &Path,
    after: &Frame,
    rect: Option<[f64; 4]>,
    viewport: (u32, u32),
) -> Result<Option<Window>> {
    let a = load(before_dir, &before.file)?;
    let b = load(after_dir, &after.file)?;
    let mut boxes: Vec<BBox> = Vec::new();
    if let Some(r) = rect {
        boxes.push(scaled_rect(r, &a, viewport));
    }
    if let Some(d) = diff_bbox(&a, &b) {
        boxes.push(d);
    }
    Ok(window_around(&boxes, a.width(), a.height()))
}

/// Crop `img` to `window` (if any) and ring `rect` in `color`.
fn framed(
    img: RgbaImage,
    rect: Option<[f64; 4]>,
    color: Rgba<u8>,
    viewport: (u32, u32),
    window: Option<Window>,
) -> RgbaImage {
    let r = rect.map(|r| scaled_rect(r, &img, viewport));
    let (mut out, ox, oy) = match window {
        Some((x, y, w, h)) => {
            let w = w.min(img.width().saturating_sub(x)).max(1);
            let h = h.min(img.height().saturating_sub(y)).max(1);
            (
                image::imageops::crop_imm(&img, x, y, w, h).to_image(),
                x as f64,
                y as f64,
            )
        }
        None => (img, 0.0, 0.0),
    };
    if let Some((rx, ry, rw, rh)) = r {
        border(
            &mut out,
            (rx - ox - 8.0) as i64,
            (ry - oy - 8.0) as i64,
            (rw + 16.0) as i64,
            (rh + 16.0) as i64,
            4,
            color,
        );
    }
    out
}

fn fit_width(img: RgbaImage, width: u32) -> RgbaImage {
    let h = (img.height() as f64 * width as f64 / img.width() as f64).round() as u32;
    image::imageops::resize(&img, width, h.max(1), FilterType::Triangle)
}

/// Build the horizontal strip. One crop window is used for every panel so the
/// panels line up; the last panel gets a verdict-coloured frame and a ring around
/// the element that triggered the behaviour.
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

    let imgs: Vec<RgbaImage> = picked
        .iter()
        .map(|f| load(dir, &f.file))
        .collect::<Result<_>>()?;
    let trig = trigger_rect(frames);
    let mut boxes: Vec<BBox> = Vec::new();
    if let Some(r) = trig {
        boxes.push(scaled_rect(r, &imgs[0], viewport));
    }
    if imgs.len() >= 2 {
        if let Some(d) = diff_bbox(&imgs[0], imgs.last().unwrap()) {
            boxes.push(d);
        }
    }
    let window = window_around(&boxes, imgs[0].width(), imgs[0].height());

    let last = imgs.len() - 1;
    let mut thumbs: Vec<RgbaImage> = Vec::new();
    for (i, img) in imgs.into_iter().enumerate() {
        // The ring only goes on the final panel.
        let ring = if i == last { trig } else { None };
        thumbs.push(fit_width(
            framed(img, ring, verdict_color(kind), viewport, window),
            thumb_w,
        ));
    }

    let max_h = thumbs.iter().map(|t| t.height()).max().unwrap_or(1);
    let total_w = margin * 2 + thumbs.len() as u32 * thumb_w + (thumbs.len() as u32 - 1) * gutter;
    let mut canvas = RgbaImage::from_pixel(total_w, margin * 2 + max_h, PAPER);
    for (i, t) in thumbs.iter().enumerate() {
        let x0 = margin + i as u32 * (thumb_w + gutter);
        image::imageops::overlay(&mut canvas, t, x0 as i64, margin as i64);
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

/// One cropped frame with a ring around `rect`, for the before/after slider.
pub fn compose_single(
    dir: &Path,
    frame: &Frame,
    rect: Option<[f64; 4]>,
    color: Rgba<u8>,
    viewport: (u32, u32),
    window: Option<Window>,
    out: &Path,
) -> Result<()> {
    framed(load(dir, &frame.file)?, rect, color, viewport, window)
        .save(out)
        .with_context(|| format!("writing {}", out.display()))?;
    Ok(())
}

/// Before/after panel: two frames cropped to the same window, red vs green.
#[allow(clippy::too_many_arguments)]
pub fn compose_compare(
    before_dir: &Path,
    before: &Frame,
    before_rect: Option<[f64; 4]>,
    after_dir: &Path,
    after: &Frame,
    viewport: (u32, u32),
    window: Option<Window>,
    out: &Path,
) -> Result<()> {
    let thumb_w: u32 = 760;
    let gutter: u32 = 24;
    let margin: u32 = 28;
    let a = fit_width(
        framed(
            load(before_dir, &before.file)?,
            before_rect,
            RED,
            viewport,
            window,
        ),
        thumb_w,
    );
    let b = fit_width(
        framed(
            load(after_dir, &after.file)?,
            before_rect,
            GREEN,
            viewport,
            window,
        ),
        thumb_w,
    );
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

    fn real_frames() -> (tempfile::TempDir, evidence::Extracted) {
        let pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/coupon-bug.evidence");
        let dir = tempfile::tempdir().unwrap();
        let ex = evidence::extract(&pack, dir.path(), "").unwrap();
        (dir, ex)
    }

    #[test]
    fn composes_strip_from_real_pack() {
        let (dir, ex) = real_frames();
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
        let (dir, ex) = real_frames();
        let first = &ex.frames[1];
        let last = ex.frames.last().unwrap();
        let rect = trigger_rect(&ex.frames);
        let vp = ex.viewport.unwrap();
        let window = change_window(dir.path(), first, dir.path(), last, rect, vp).unwrap();
        assert!(window.is_some());
        let out = std::env::temp_dir().join("reenact-compare-preview.png");
        compose_compare(dir.path(), last, rect, dir.path(), last, vp, window, &out).unwrap();
        let img = image::open(&out).unwrap();
        assert!(img.height() < 700, "height {}", img.height());
    }

    #[test]
    fn diff_bbox_finds_what_changed() {
        let a = RgbaImage::from_pixel(400, 300, Rgba([255, 255, 255, 255]));
        let mut b = a.clone();
        for y in 100..140 {
            for x in 200..320 {
                b.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let (x, y, w, h) = diff_bbox(&a, &b).unwrap();
        assert!((195.0..=205.0).contains(&x) && (95.0..=105.0).contains(&y));
        assert!((110.0..=125.0).contains(&w) && (35.0..=45.0).contains(&h));
        assert!(diff_bbox(&a, &a).is_none());
    }

    #[test]
    fn windows_stay_inside_the_image() {
        let (x, y, w, h) = window_around(&[(1800.0, 1000.0, 100.0, 60.0)], 1920, 1080).unwrap();
        assert!(x + w <= 1920 && y + h <= 1080);
        let (x, y, w, h) = window_around(&[(0.0, 0.0, 10.0, 10.0)], 1920, 1080).unwrap();
        assert!(x == 0 && y == 0 && w >= 640 && h >= 360);
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
