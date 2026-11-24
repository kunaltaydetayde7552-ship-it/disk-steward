//! Menu bar glyph drawn at runtime with tiny-skia: two plates pressing a disk. Geometry is in points,
//! rendered at 2x for the macOS menu bar (template, black) and 1x white elsewhere.

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

const W_PT: f32 = 20.0;
const H_PT: f32 = 16.0;
/// `(x, y, w, h, alpha)` of the three bars, in points.
const BARS: [(f32, f32, f32, f32, f32); 3] = [
    (1.0, 1.5, 18.0, 4.0, 1.0),
    (4.0, 6.5, 12.0, 3.0, 0.55),
    (1.0, 10.5, 18.0, 4.0, 1.0),
];
const RADIUS: f32 = 1.8;

pub struct Glyph {
    /// Straight (non-premultiplied) RGBA.
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

/// `template` = macOS (2x, black, the system tints it); otherwise 1x white.
pub fn render(template: bool) -> Glyph {
    let scale = if template { 2.0 } else { 1.0 };
    let (w, h) = ((W_PT * scale) as u32, (H_PT * scale) as u32);
    let mut pm = Pixmap::new(w, h).expect("non-zero glyph canvas");
    let shade = if template { 0 } else { 255 };
    for (x, y, bw, bh, alpha) in BARS {
        let mut paint = Paint::default();
        paint.set_color_rgba8(shade, shade, shade, (alpha * 255.0) as u8);
        paint.anti_alias = true;
        if let Some(path) =
            rounded_rect(x * scale, y * scale, bw * scale, bh * scale, RADIUS * scale)
        {
            pm.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }
    Glyph {
        rgba: unpremultiply(&pm),
        w,
        h,
    }
}

fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = r * 0.552_284_8; // cubic approximation of a quarter circle
    let mut p = PathBuilder::new();
    p.move_to(x + r, y);
    p.line_to(x + w - r, y);
    p.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    p.line_to(x + w, y + h - r);
    p.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    p.line_to(x + r, y + h);
    p.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    p.line_to(x, y + r);
    p.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    p.close();
    p.finish()
}

fn unpremultiply(pm: &Pixmap) -> Vec<u8> {
    pm.pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_is_2x_black_with_transparent_corners() {
        let g = render(true);
        assert_eq!((g.w, g.h, g.rgba.len()), (40, 32, 40 * 32 * 4));
        assert_eq!(g.rgba[3], 0); // top-left pixel is outside the first bar
        let solid: Vec<_> = g.rgba.chunks(4).filter(|p| p[3] == 255).collect();
        assert!(!solid.is_empty() && solid.iter().all(|p| p[..3] == [0, 0, 0]));
    }

    #[test]
    fn non_template_is_1x_white() {
        let g = render(false);
        assert_eq!((g.w, g.h), (20, 16));
        assert!(
            g.rgba
                .chunks(4)
                .filter(|p| p[3] == 255)
                .all(|p| p[..3] == [255, 255, 255])
        );
    }
}
