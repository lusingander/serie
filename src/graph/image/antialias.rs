use image::{Rgba, RgbaImage};
use rustc_hash::FxHashMap;

use crate::graph::{
    geometry::Point,
    image::{
        bezier_point, curved_connection_points, diagonal_connection, DrawingPixels, GraphStyle,
        ImageParams, Pixels,
    },
    Edge, EdgeType,
};

const SAMPLE_GRID: usize = 4;
const SAMPLE_COUNT: usize = SAMPLE_GRID * SAMPLE_GRID;
const FULL_COVERAGE: u16 = u16::MAX;

// Keep sample positions, rather than just their count, so overlapping pieces
// of a connection and adjacent fill/outline regions can be combined exactly.
type Coverage = FxHashMap<(i32, i32), u16>;

#[derive(Debug)]
pub struct Masks {
    circle: Coverage,
    circle_edge: Coverage,
    corners: [Coverage; 4],
}

impl Masks {
    pub fn new(params: &ImageParams) -> Self {
        let center = Point::new(
            f64::from(params.width / 2) + 0.5,
            f64::from(params.height / 2) + 0.5,
        );
        let bounds = [
            Point::new(0.0, 0.0),
            Point::new(f64::from(params.width), f64::from(params.height)),
        ];
        let mut circle = Coverage::default();
        let mut outer = Coverage::default();
        for (radius, mask) in [
            (params.circle_inner_radius, &mut circle),
            (params.circle_outer_radius, &mut outer),
        ] {
            let radius = f64::from(radius) + 0.5;
            sample_shape(mask, bounds, |p| {
                (p - center).dot(p - center) <= radius * radius
            });
        }
        for (pos, mask) in &mut outer {
            *mask &= !circle.get(pos).copied().unwrap_or_default();
        }
        outer.retain(|_, mask| *mask != 0);
        let corners = [
            EdgeType::RightTop,
            EdgeType::LeftTop,
            EdgeType::RightBottom,
            EdgeType::LeftBottom,
        ]
        .map(|edge_type| {
            let mut mask = Coverage::default();
            sample_shape(&mut mask, bounds, |p| {
                rounded_corner_contains(p, edge_type, params)
            });
            mask
        });
        Self {
            circle,
            circle_edge: outer,
            corners,
        }
    }
}

fn rounded_corner_contains(p: Point, edge_type: EdgeType, params: &ImageParams) -> bool {
    let offset = f64::from(params.line_width % 2) / 2.0;
    let center = Point::new(
        f64::from(params.width / 2) + offset,
        f64::from(params.height / 2) + offset,
    );
    let radius = f64::from(params.corner_radius());
    let half_width = f64::from(params.line_width) / 2.0;
    let right = matches!(edge_type, EdgeType::RightTop | EdgeType::RightBottom);
    let bottom = matches!(edge_type, EdgeType::RightBottom | EdgeType::LeftBottom);
    let corner_center = Point::new(
        center.x + if right { -radius } else { radius },
        center.y + if bottom { -radius } else { radius },
    );
    let in_x = if right {
        p.x >= corner_center.x
    } else {
        p.x <= corner_center.x
    };
    let in_y = if bottom {
        p.y >= corner_center.y
    } else {
        p.y <= corner_center.y
    };
    if !in_x {
        return (p.y - center.y).abs() <= half_width;
    }
    if !in_y {
        return (p.x - center.x).abs() <= half_width;
    }
    let distance = (p - corner_center).length();
    (radius - half_width).max(0.0) <= distance && distance <= radius + half_width
}

pub fn render(
    commit_pos_x: usize,
    cell_count: usize,
    edges: &[Edge],
    params: &ImageParams,
    pixels: &DrawingPixels,
    style: GraphStyle,
) -> RgbaImage {
    let background = if params.background_color[3] == 0 {
        Rgba([0; 4])
    } else {
        params.background_color
    };
    let mut image = SampledImage::new(
        (cell_count * params.width as usize) as u32,
        u32::from(params.height),
        background,
    );
    let masks = pixels.antialias.as_ref().unwrap();
    let offset = (commit_pos_x * params.width as usize) as i32;
    image.paint(&masks.circle, offset, params.edge_color(commit_pos_x));
    if params.circle_edge_color[3] != 0 {
        image.paint(&masks.circle_edge, offset, params.circle_edge_color);
    }
    let mut ordered_edges: Vec<_> = edges.iter().collect();
    ordered_edges.sort_by_key(|e| (e.associated_line_pos_x, e.pos_x, e.edge_type));
    for edge in &ordered_edges {
        if style != GraphStyle::Rounded && !edge.edge_type.is_vertically_related() {
            continue;
        }
        let offset = (edge.pos_x * params.width as usize) as i32;
        let color = params.edge_color(edge.associated_line_pos_x);
        match edge.edge_type {
            EdgeType::RightTop => image.paint(&masks.corners[0], offset, color),
            EdgeType::LeftTop => image.paint(&masks.corners[1], offset, color),
            EdgeType::RightBottom => image.paint(&masks.corners[2], offset, color),
            EdgeType::LeftBottom => image.paint(&masks.corners[3], offset, color),
            t => image.paint_pixels(
                match t {
                    EdgeType::Vertical => &pixels.vertical_edge,
                    EdgeType::Horizontal => &pixels.horizontal_edge,
                    EdgeType::Up => &pixels.up_edge,
                    EdgeType::Down => &pixels.down_edge,
                    EdgeType::Left => &pixels.left_edge,
                    EdgeType::Right => &pixels.right_edge,
                    _ => unreachable!(),
                },
                offset,
                color,
            ),
        }
    }
    if style != GraphStyle::Rounded {
        for corner in ordered_edges {
            let side_type = match corner.edge_type {
                EdgeType::RightTop | EdgeType::RightBottom => EdgeType::Right,
                EdgeType::LeftTop | EdgeType::LeftBottom => EdgeType::Left,
                _ => continue,
            };
            let Some(side) = edges.iter().find(|e| {
                e.edge_type == side_type && e.associated_line_pos_x == corner.associated_line_pos_x
            }) else {
                continue;
            };
            let coverage = match style {
                GraphStyle::Angular => diagonal_coverage(side, corner, params),
                GraphStyle::Curved => curved_coverage(side, corner, params),
                GraphStyle::Rounded => unreachable!(),
            };
            image.paint(
                &coverage,
                0,
                params.edge_color(corner.associated_line_pos_x),
            );
        }
    }
    image.resolve()
}

fn diagonal_coverage(side: &Edge, corner: &Edge, params: &ImageParams) -> Coverage {
    let (vertices, ys, x_start) = diagonal_connection(side, corner, params);
    let bounds = [
        Point::new(
            vertices.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
            vertices.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
        ),
        Point::new(
            vertices
                .iter()
                .map(|p| p.x)
                .fold(f64::NEG_INFINITY, f64::max),
            vertices
                .iter()
                .map(|p| p.y)
                .fold(f64::NEG_INFINITY, f64::max),
        ),
    ];
    let mut coverage = Coverage::default();
    sample_shape(&mut coverage, bounds, |p| p.is_inside_polygon(&vertices));
    for y in ys {
        for x in x_start..x_start + i32::from(params.line_width) {
            coverage.insert((x, y as i32), FULL_COVERAGE);
        }
    }
    coverage
}

fn curved_coverage(side: &Edge, corner: &Edge, params: &ImageParams) -> Coverage {
    let points = curved_connection_points(side, corner, params);
    let radius = f64::from(params.line_width) / 2.0;
    let direction = (points[3].x - points[0].x).signum();
    let boundary_y = if points[3].y == 0.0 {
        0
    } else {
        i32::from(params.height) - 1
    };
    let steps = ((((points[3].x - points[0].x).abs() + (points[3].y - points[0].y).abs()) * 3.0)
        .ceil() as usize)
        .max(32);
    let mut coverage = Coverage::default();
    let mut start = points[0];
    for step in 1..=steps {
        let end = bezier_point(&points, step as f64 / steps as f64);
        let segment = end - start;
        let length_squared = segment.dot(segment);
        let bounds = [
            Point::new(
                (start.x.min(end.x) - radius).max(0.0),
                (start.y.min(end.y) - radius).max(0.0),
            ),
            Point::new(
                end.x.max(start.x) + radius,
                (end.y.max(start.y) + radius).min(f64::from(params.height)),
            ),
        ];
        sample_shape(&mut coverage, bounds, |p| {
            if p.y.floor() as i32 == boundary_y || (p.x - points[0].x) * direction < 0.0 {
                return false;
            }
            let t = if length_squared == 0.0 {
                0.0
            } else {
                ((p - start).dot(segment) / length_squared).clamp(0.0, 1.0)
            };
            let delta = p - (start + segment * t);
            delta.dot(delta) <= radius * radius
        });
        start = end;
    }
    // Preserve the straight-edge mask at the boundary shared with the next row.
    let x_start = (points[3].x - radius) as i32;
    for x in x_start..x_start + i32::from(params.line_width) {
        coverage.insert((x, boundary_y), FULL_COVERAGE);
    }
    coverage
}

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

    fn paint_pixels(&mut self, pixels: &Pixels, x_offset: i32, color: Rgba<u8>) {
        for &(x, y) in pixels {
            let x = x + x_offset;
            if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
                self.pixels[(y as u32 * self.width + x as u32) as usize] = [color; SAMPLE_COUNT];
            }
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
