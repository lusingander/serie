use image::{Rgba, RgbaImage};
use rustc_hash::FxHashMap;

use crate::graph::geometry::Point;

const SAMPLE_GRID: usize = 4;
const SAMPLE_COUNT: usize = SAMPLE_GRID * SAMPLE_GRID;
const FULL_COVERAGE: u16 = u16::MAX;

// Keep sample positions, rather than just their count, so overlapping pieces
// of a connection and adjacent fill/outline regions can be combined exactly.
type Coverage = FxHashMap<(i32, i32), u16>;

fn sample_shape(coverage: &mut Coverage, bounds: [Point; 2], contains: impl Fn(Point) -> bool) {
    for y in bounds[0].y.floor() as i32..bounds[1].y.ceil() as i32 {
        for x in bounds[0].x.floor() as i32..bounds[1].x.ceil() as i32 {
            let mut mask = 0;
            for sy in 0..SAMPLE_GRID {
                for sx in 0..SAMPLE_GRID {
                    let p = Point::new(
                        f64::from(x) + (sx as f64 + 0.5) / SAMPLE_GRID as f64,
                        f64::from(y) + (sy as f64 + 0.5) / SAMPLE_GRID as f64,
                    );
                    if contains(p) {
                        mask |= 1 << (sy * SAMPLE_GRID + sx);
                    }
                }
            }
            if mask != 0 {
                *coverage.entry((x, y)).or_default() |= mask;
            }
        }
    }
}

struct SampledImage {
    width: u32,
    height: u32,
    pixels: Vec<[Rgba<u8>; SAMPLE_COUNT]>,
}

impl SampledImage {
    fn new(width: u32, height: u32, background: Rgba<u8>) -> Self {
        Self {
            width,
            height,
            pixels: vec![[background; SAMPLE_COUNT]; (width * height) as usize],
        }
    }

    fn paint(&mut self, coverage: &Coverage, x_offset: i32, color: Rgba<u8>) {
        for (&(x, y), &mask) in coverage {
            let x = x + x_offset;
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                continue;
            }
            let samples = &mut self.pixels[(y as u32 * self.width + x as u32) as usize];
            for (i, sample) in samples.iter_mut().enumerate() {
                if mask & (1 << i) != 0 {
                    // Preserve the existing renderer's replacement semantics,
                    // including when a configured color is translucent.
                    *sample = color;
                }
            }
        }
    }

    fn resolve(self) -> RgbaImage {
        RgbaImage::from_fn(self.width, self.height, |x, y| {
            let samples = &self.pixels[(y * self.width + x) as usize];
            if samples.iter().all(|p| p == &samples[0]) {
                return samples[0];
            }
            let alpha: u32 = samples.iter().map(|p| u32::from(p[3])).sum();
            if alpha == 0 {
                return Rgba([0; 4]);
            }
            let mut color = [0; 4];
            // Average premultiplied colors, then return straight RGBA for PNG.
            for channel in 0..3 {
                let sum: u32 = samples
                    .iter()
                    .map(|p| u32::from(p[channel]) * u32::from(p[3]))
                    .sum();
                color[channel] = ((sum + alpha / 2) / alpha) as u8;
            }
            color[3] = ((alpha + SAMPLE_COUNT as u32 / 2) / SAMPLE_COUNT as u32) as u8;
            Rgba(color)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_preserves_color_and_combines_overlapping_samples() {
        let mut mask = Coverage::default();
        let bounds = [Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
        sample_shape(&mut mask, bounds, |p| p.x < 0.5);
        let color = Rgba([200, 80, 40, 128]);
        let mut image = SampledImage::new(1, 1, Rgba([0; 4]));
        image.paint(&mask, 0, color);
        image.paint(&mask, 0, color);
        assert_eq!(*image.resolve().get_pixel(0, 0), Rgba([200, 80, 40, 64]));

        sample_shape(&mut mask, bounds, |p| p.x >= 0.5);
        assert_eq!(mask[&(0, 0)], FULL_COVERAGE);
        let mut image = SampledImage::new(1, 1, Rgba([0, 255, 0, 112]));
        image.paint(&mask, 0, color);
        assert_eq!(*image.resolve().get_pixel(0, 0), color);
    }

    #[test]
    fn adjacent_fill_and_outline_do_not_leave_a_transparent_seam() {
        let mut fill = Coverage::default();
        let bounds = [Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
        sample_shape(&mut fill, bounds, |p| p.x < 0.5);
        let outline = Coverage::from_iter([((0, 0), FULL_COVERAGE & !fill[&(0, 0)])]);
        let mut image = SampledImage::new(1, 1, Rgba([0; 4]));
        image.paint(&fill, 0, Rgba([255, 0, 0, 255]));
        image.paint(&outline, 0, Rgba([255; 4]));
        assert_eq!(*image.resolve().get_pixel(0, 0), Rgba([255, 128, 128, 255]));
    }
}
