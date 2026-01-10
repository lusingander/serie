use std::{
    fmt::{self, Debug, Formatter},
    io::Cursor,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    color::GraphColorSet,
    git::CommitHash,
    graph::{
        geometry::{bounding_box_u32, Point},
        Edge, EdgeType, Graph,
    },
    protocol::ImageProtocol,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphStyle {
    Rounded,
    Angular,
}

#[derive(Debug)]
pub struct GraphImageManager<'a> {
    encoded_image_map: FxHashMap<CommitHash, String>,

    graph: &'a Graph<'a>,
    cell_width_type: CellWidthType,
    graph_style: GraphStyle,
    image_params: ImageParams,
    drawing_pixels: DrawingPixels,
    image_protocol: ImageProtocol,
}

impl<'a> GraphImageManager<'a> {
    pub fn new(
        graph: &'a Graph,
        graph_color_set: &GraphColorSet,
        cell_width_type: CellWidthType,
        graph_style: GraphStyle,
        image_protocol: ImageProtocol,
        preload: bool,
    ) -> Self {
        let image_params = ImageParams::new(graph_color_set, cell_width_type);
        let drawing_pixels = DrawingPixels::new(&image_params);

        let mut m = GraphImageManager {
            encoded_image_map: FxHashMap::default(),
            graph,
            cell_width_type,
            graph_style,
            image_params,
            drawing_pixels,
            image_protocol,
        };
        if preload {
            m.load_all_encoded_image();
        }
        m
    }

    pub fn encoded_image(&self, commit_hash: &CommitHash) -> &str {
        self.encoded_image_map.get(commit_hash).unwrap()
    }

    pub fn load_all_encoded_image(&mut self) {
        let graph_image = build_graph_image(
            self.graph,
            &self.image_params,
            &self.drawing_pixels,
            self.graph_style,
        );
        self.encoded_image_map = self
            .graph
            .commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let edges = &self.graph.edges[i];
                let image =
                    graph_image.images[edges].encode(self.cell_width_type, self.image_protocol);
                (commit.commit_hash.clone(), image)
            })
            .collect()
    }

    pub fn load_encoded_image(&mut self, commit_hash: &CommitHash) {
        if self.encoded_image_map.contains_key(commit_hash) {
            return;
        }
        let graph_row_image = build_single_graph_row_image(
            self.graph,
            &self.image_params,
            &self.drawing_pixels,
            self.graph_style,
            commit_hash,
        );
        let image = graph_row_image.encode(self.cell_width_type, self.image_protocol);
        self.encoded_image_map.insert(commit_hash.clone(), image);
    }
}

#[derive(Debug, Default)]
pub struct GraphImage {
    pub images: FxHashMap<Vec<Edge>, GraphRowImage>,
}

pub struct GraphRowImage {
    pub bytes: Vec<u8>,
    pub cell_count: usize,
}

impl Debug for GraphRowImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GraphRowImage {{ bytes: [{} bytes], cell_count: {} }}",
            self.bytes.len(),
            self.cell_count
        )
    }
}

impl GraphRowImage {
    fn encode(&self, cell_width_type: CellWidthType, image_protocol: ImageProtocol) -> String {
        let image_cell_width = match cell_width_type {
            CellWidthType::Double => self.cell_count * 2,
            CellWidthType::Single => self.cell_count,
        };
        image_protocol.encode(&self.bytes, image_cell_width)
    }
}

#[derive(Debug)]
pub struct ImageParams {
    width: u16,
    height: u16,
    line_width: u16,
    circle_inner_radius: u16,
    circle_outer_radius: u16,
    edge_colors: Vec<image::Rgba<u8>>,
    circle_edge_color: image::Rgba<u8>,
    background_color: image::Rgba<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellWidthType {
    Double, // 2 cells
    Single,
}

impl ImageParams {
    pub fn new(graph_color_set: &GraphColorSet, cell_width_type: CellWidthType) -> Self {
        let (width, height, line_width, circle_inner_radius, circle_outer_radius) =
            match cell_width_type {
                CellWidthType::Double => (50, 50, 5, 10, 13),
                CellWidthType::Single => (25, 50, 3, 7, 10),
            };
        let edge_colors = graph_color_set
            .colors
            .iter()
            .map(|c| c.to_image_color())
            .collect();
        let circle_edge_color = graph_color_set.edge_color.to_image_color();
        let background_color = graph_color_set.background_color.to_image_color();
        Self {
            width,
            height,
            line_width,
            circle_inner_radius,
            circle_outer_radius,
            edge_colors,
            circle_edge_color,
            background_color,
        }
    }

    fn edge_color(&self, index: usize) -> image::Rgba<u8> {
        self.edge_colors[index % self.edge_colors.len()]
    }

    fn corner_radius(&self) -> u16 {
        if self.width < self.height {
            self.width / 2
        } else {
            self.height / 2
        }
    }
}

fn build_single_graph_row_image(
    graph: &Graph<'_>,
    image_params: &ImageParams,
    drawing_pixels: &DrawingPixels,
    graph_style: GraphStyle,
    commit_hash: &CommitHash,
) -> GraphRowImage {
    let (pos_x, pos_y) = graph.commit_pos_map[&commit_hash];
    let edges = &graph.edges[pos_y];

    let cell_count = graph.max_pos_x + 1;

    calc_graph_row_image(
        pos_x,
        cell_count,
        edges,
        image_params,
        drawing_pixels,
        graph_style,
    )
}

pub fn build_graph_image(
    graph: &Graph<'_>,
    image_params: &ImageParams,
    drawing_pixels: &DrawingPixels,
    graph_style: GraphStyle,
) -> GraphImage {
    let graph_row_sources: FxHashSet<(usize, &Vec<Edge>)> = graph
        .commits
        .iter()
        .map(|commit| {
            let (pos_x, pos_y) = graph.commit_pos_map[&commit.commit_hash];
            let edges = &graph.edges[pos_y];
            (pos_x, edges)
        })
        .collect();

    let cell_count = graph.max_pos_x + 1;

    let images = graph_row_sources
        .into_par_iter()
        .map(|(pos_x, edges)| {
            let graph_row_image = calc_graph_row_image(
                pos_x,
                cell_count,
                edges,
                image_params,
                drawing_pixels,
                graph_style,
            );
            (edges.clone(), graph_row_image)
        })
        .collect();

    GraphImage { images }
}

type Pixels = FxHashSet<(i32, i32)>;

#[derive(Debug)]
pub struct DrawingPixels {
    circle: Pixels,
    circle_edge: Pixels,
    vertical_edge: Pixels,
    horizontal_edge: Pixels,
    up_edge: Pixels,
    down_edge: Pixels,
    left_edge: Pixels,
    right_edge: Pixels,
    right_top_edge: Pixels,
    left_top_edge: Pixels,
    right_bottom_edge: Pixels,
    left_bottom_edge: Pixels,
}

impl DrawingPixels {
    pub fn new(image_params: &ImageParams) -> Self {
        let circle = calc_commit_circle_drawing_pixels(image_params);
        let circle_edge = calc_circle_edge_drawing_pixels(image_params);
        let vertical_edge = calc_vertical_edge_drawing_pixels(image_params);
        let horizontal_edge = calc_horizontal_edge_drawing_pixels(image_params);
        let up_edge = calc_up_edge_drawing_pixels(image_params);
        let down_edge = calc_down_edge_drawing_pixels(image_params);
        let left_edge = calc_left_edge_drawing_pixels(image_params);
        let right_edge = calc_right_edge_drawing_pixels(image_params);
        let right_top_edge = calc_right_top_edge_drawing_pixels(image_params);
        let left_top_edge = calc_left_top_edge_drawing_pixels(image_params);
        let right_bottom_edge = calc_right_bottom_edge_drawing_pixels(image_params);
        let left_bottom_edge = calc_left_bottom_edge_drawing_pixels(image_params);

        Self {
            circle,
            circle_edge,
            vertical_edge,
            horizontal_edge,
            up_edge,
            down_edge,
            left_edge,
            right_edge,
            right_top_edge,
            left_top_edge,
            right_bottom_edge,
            left_bottom_edge,
        }
    }
}

fn calc_commit_circle_drawing_pixels(image_params: &ImageParams) -> Pixels {
    calc_circle_drawing_pixels(image_params, image_params.circle_inner_radius as i32)
}

fn calc_circle_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let inner = calc_circle_drawing_pixels(image_params, image_params.circle_inner_radius as i32);
    let outer = calc_circle_drawing_pixels(image_params, image_params.circle_outer_radius as i32);

    outer.difference(&inner).cloned().collect()
}

fn calc_circle_drawing_pixels(image_params: &ImageParams, radius: i32) -> Pixels {
    // Bresenham's circle algorithm
    let center_x = (image_params.width / 2) as i32;
    let center_y = (image_params.height / 2) as i32;

    let mut x = radius;
    let mut y = 0;
    let mut p = 1 - radius;

    let mut pixels = Pixels::default();

    while x >= y {
        for dx in -x..=x {
            pixels.insert((center_x + dx, center_y + y));
            pixels.insert((center_x + dx, center_y - y));
        }
        for dx in -y..=y {
            pixels.insert((center_x + dx, center_y + x));
            pixels.insert((center_x + dx, center_y - x));
        }

        y += 1;
        if p <= 0 {
            p += 2 * y + 1;
        } else {
            x -= 1;
            p += 2 * y - 2 * x + 1;
        }
    }

    pixels
}

fn calc_vertical_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_x = (image_params.width / 2) as i32;
    let line_width = image_params.line_width as i32;
    let x_start = center_x - line_width / 2;

    let mut pixels = Pixels::default();
    for y in 0..image_params.height as i32 {
        for x in x_start..(x_start + line_width) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_horizontal_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_y = (image_params.height / 2) as i32;
    let line_width = image_params.line_width as i32;
    let y_start = center_y - line_width / 2;

    let mut pixels = Pixels::default();
    for y in y_start..(y_start + line_width) {
        for x in 0..image_params.width as i32 {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_up_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_x = (image_params.width / 2) as i32;
    let line_width = image_params.line_width as i32;
    let x_start = center_x - line_width / 2;
    let circle_center_y = (image_params.height / 2) as i32;
    let circle_outer_radius = image_params.circle_outer_radius as i32;

    let mut pixels = Pixels::default();
    for y in 0..(circle_center_y - circle_outer_radius) {
        for x in x_start..(x_start + line_width) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_down_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_x = (image_params.width / 2) as i32;
    let line_width = image_params.line_width as i32;
    let x_start = center_x - line_width / 2;
    let circle_center_y = (image_params.height / 2) as i32;
    let circle_outer_radius = image_params.circle_outer_radius as i32;

    let mut pixels = Pixels::default();
    for y in (circle_center_y + circle_outer_radius + 1)..(image_params.height as i32) {
        for x in x_start..(x_start + line_width) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_left_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_y = (image_params.height / 2) as i32;
    let line_width = image_params.line_width as i32;
    let y_start = center_y - line_width / 2;
    let circle_center_x = (image_params.width / 2) as i32;
    let circle_outer_radius = image_params.circle_outer_radius as i32;

    let mut pixels = Pixels::default();
    for y in y_start..(y_start + line_width) {
        for x in 0..(circle_center_x - circle_outer_radius) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_right_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_y = (image_params.height / 2) as i32;
    let line_width = image_params.line_width as i32;
    let y_start = center_y - line_width / 2;
    let circle_center_x = (image_params.width / 2) as i32;
    let circle_outer_radius = image_params.circle_outer_radius as i32;

    let mut pixels = Pixels::default();
    for y in y_start..(y_start + line_width) {
        for x in (circle_center_x + circle_outer_radius + 1)..(image_params.width as i32) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_right_top_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let (w, h, r) = (
        image_params.width as i32,
        image_params.height as i32,
        image_params.corner_radius() as i32,
    );
    let (x_offset, y_offset) = if w < h {
        (0, r - (h / 2))
    } else {
        ((w / 2) - r, 0)
    };
    calc_corner_edge_drawing_pixels(image_params, 0, h, x_offset, y_offset)
}

fn calc_left_top_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let (w, h, r) = (
        image_params.width as i32,
        image_params.height as i32,
        image_params.corner_radius() as i32,
    );
    let (x_offset, y_offset) = if w < h {
        (0, r - (h / 2))
    } else {
        (r - (w / 2), 0)
    };
    calc_corner_edge_drawing_pixels(image_params, w, h, x_offset, y_offset)
}

fn calc_right_bottom_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let (w, h, r) = (
        image_params.width as i32,
        image_params.height as i32,
        image_params.corner_radius() as i32,
    );
    let (x_offset, y_offset) = if w < h {
        (0, (h / 2) - r)
    } else {
        ((w / 2) - r, 0)
    };
    calc_corner_edge_drawing_pixels(image_params, 0, 0, x_offset, y_offset)
}

fn calc_left_bottom_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let (w, h, r) = (
        image_params.width as i32,
        image_params.height as i32,
        image_params.corner_radius() as i32,
    );
    let (x_offset, y_offset) = if w < h {
        (0, (h / 2) - r)
    } else {
        (r - (w / 2), 0)
    };
    calc_corner_edge_drawing_pixels(image_params, w, 0, x_offset, y_offset)
}

fn calc_corner_edge_drawing_pixels(
    image_params: &ImageParams,
    base_center_x: i32,
    base_center_y: i32,
    x_offset: i32,
    y_offset: i32,
) -> Pixels {
    // Bresenham's circle algorithm
    let curve_center_x = base_center_x;
    let curve_center_y = base_center_y;
    let line_width = image_params.line_width as i32;
    let half_line_width = line_width / 2;
    let adjust = if image_params.line_width.is_multiple_of(2) {
        0
    } else {
        1
    };
    let radius_base_length = image_params.corner_radius() as i32;
    let inner_radius = radius_base_length - half_line_width - adjust;
    let outer_radius = radius_base_length + half_line_width;

    let mut x = inner_radius;
    let mut y = 0;
    let mut p = 1 - inner_radius;

    let mut inner_pixels = Pixels::default();

    while x >= y {
        for dx in -x..=x {
            inner_pixels.insert((curve_center_x + dx, curve_center_y + y));
            inner_pixels.insert((curve_center_x + dx, curve_center_y - y));
        }
        for dx in -y..=y {
            inner_pixels.insert((curve_center_x + dx, curve_center_y + x));
            inner_pixels.insert((curve_center_x + dx, curve_center_y - x));
        }

        y += 1;
        if p <= 0 {
            p += 2 * y + 1;
        } else {
            x -= 1;
            p += 2 * y - 2 * x + 1;
        }
    }

    let mut x = outer_radius;
    let mut y = 0;
    let mut p = 1 - outer_radius;

    let mut outer_pixels = Pixels::default();

    while x >= y {
        for dx in -x..=x {
            outer_pixels.insert((curve_center_x + dx, curve_center_y + y));
            outer_pixels.insert((curve_center_x + dx, curve_center_y - y));
        }
        for dx in -y..=y {
            outer_pixels.insert((curve_center_x + dx, curve_center_y + x));
            outer_pixels.insert((curve_center_x + dx, curve_center_y - x));
        }

        y += 1;
        if p <= 0 {
            p += 2 * y + 1;
        } else {
            x -= 1;
            p += 2 * y - 2 * x + 1;
        }
    }

    let mut pixels: Pixels = outer_pixels
        .difference(&inner_pixels)
        .filter(|p| {
            p.0 >= 0
                && p.0 < image_params.width as i32
                && p.1 >= 0
                && p.1 < image_params.height as i32
        })
        .map(|p| (p.0 + x_offset, p.1 + y_offset))
        .collect();

    if image_params.width < image_params.height {
        let (ys, ye) = if y_offset < 0 {
            (base_center_y + y_offset, base_center_y)
        } else {
            (base_center_y, base_center_y + y_offset)
        };
        let center_x = (image_params.width / 2) as i32;
        let x_start = center_x - line_width / 2;
        for x in x_start..(x_start + line_width) {
            for y in ys..ye {
                pixels.insert((x, y));
            }
        }
    }
    if image_params.width > image_params.height {
        let (xs, xe) = if x_offset < 0 {
            (base_center_x + x_offset, base_center_x)
        } else {
            (base_center_x, base_center_x + x_offset)
        };
        let center_y = (image_params.height / 2) as i32;
        let y_start = center_y - line_width / 2;
        for y in y_start..(y_start + line_width) {
            for x in xs..xe {
                pixels.insert((x, y));
            }
        }
    }

    pixels
}

fn calc_graph_row_image(
    commit_pos_x: usize,
    cell_count: usize,
    edges: &[Edge],
    image_params: &ImageParams,
    drawing_pixels: &DrawingPixels,
    graph_style: GraphStyle,
) -> GraphRowImage {
    let image_width = (image_params.width as usize * cell_count) as u32;
    let image_height = image_params.height as u32;

    let mut img_buf = image::ImageBuffer::new(image_width, image_height);

    draw_background(&mut img_buf, image_params);
    draw_commit_circle(&mut img_buf, commit_pos_x, image_params, drawing_pixels);

    match graph_style {
        GraphStyle::Rounded => {
            for edge in edges {
                draw_edge(&mut img_buf, edge, image_params, drawing_pixels)
            }
        }
        GraphStyle::Angular => {
            let (vertial_edges, horizontal_edges): (Vec<&Edge>, Vec<&Edge>) = edges
                .iter()
                .partition(|e| e.edge_type.is_vertically_related());
            for edge in vertial_edges {
                draw_edge(&mut img_buf, edge, image_params, drawing_pixels)
            }
            let mut horizontal_edges_map: FxHashMap<usize, Vec<&Edge>> = FxHashMap::default();
            for edge in horizontal_edges {
                horizontal_edges_map
                    .entry(edge.associated_line_pos_x)
                    .or_default()
                    .push(edge);
            }
            for edges in horizontal_edges_map.values() {
                draw_diagonal_connected_edge(&mut img_buf, edges, image_params);
            }
        }
    }

    let bytes = build_image(&img_buf, image_width, image_height);

    GraphRowImage { bytes, cell_count }
}

fn draw_background(
    img_buf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    image_params: &ImageParams,
) {
    if image_params.background_color[3] == 0 {
        // If the alpha value is 0, the background is transparent, so we don't need to draw it.
        return;
    }
    for pixel in img_buf.pixels_mut() {
        *pixel = image_params.background_color;
    }
}

fn draw_commit_circle(
    img_buf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    circle_pos_x: usize,
    image_params: &ImageParams,
    drawing_pixels: &DrawingPixels,
) {
    let x_offset = (circle_pos_x * image_params.width as usize) as i32;
    let color = image_params.edge_color(circle_pos_x);

    for (x, y) in &drawing_pixels.circle {
        let x = (*x + x_offset) as u32;
        let y = *y as u32;

        let pixel = img_buf.get_pixel_mut(x, y);
        *pixel = color;
    }

    if image_params.circle_edge_color[3] == 0 {
        // If the alpha value is 0, the circle edge is transparent, so we don't need to draw it.
        return;
    }

    for (x, y) in &drawing_pixels.circle_edge {
        let x = (*x + x_offset) as u32;
        let y = *y as u32;

        let pixel = img_buf.get_pixel_mut(x, y);
        *pixel = image_params.circle_edge_color;
    }
}

fn draw_edge(
    img_buf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    edge: &Edge,
    image_params: &ImageParams,
    drawing_pixels: &DrawingPixels,
) {
    let pixels = match edge.edge_type {
        EdgeType::Vertical => &drawing_pixels.vertical_edge,
        EdgeType::Horizontal => &drawing_pixels.horizontal_edge,
        EdgeType::Up => &drawing_pixels.up_edge,
        EdgeType::Down => &drawing_pixels.down_edge,
        EdgeType::Left => &drawing_pixels.left_edge,
        EdgeType::Right => &drawing_pixels.right_edge,
        EdgeType::RightTop => &drawing_pixels.right_top_edge,
        EdgeType::RightBottom => &drawing_pixels.right_bottom_edge,
        EdgeType::LeftTop => &drawing_pixels.left_top_edge,
        EdgeType::LeftBottom => &drawing_pixels.left_bottom_edge,
    };

    let x_offset = (edge.pos_x * image_params.width as usize) as i32;
    let color = image_params.edge_color(edge.associated_line_pos_x);

    for (x, y) in pixels {
        let x = (*x + x_offset) as u32;
        let y = *y as u32;

        let pixel = img_buf.get_pixel_mut(x, y);
        *pixel = color;
    }
}

// fixme: cache edge drawing range calculations
fn draw_diagonal_connected_edge(
    img_buf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    edges: &[&Edge],
    image_params: &ImageParams,
) {
    let corner_edges = edges.iter().filter(|e| {
        matches!(
            e.edge_type,
            EdgeType::RightBottom | EdgeType::LeftBottom | EdgeType::RightTop | EdgeType::LeftTop
        )
    });

    for corner_edge in corner_edges {
        let expected_side_edge_type = match corner_edge.edge_type {
            EdgeType::RightBottom | EdgeType::RightTop => EdgeType::Right,
            EdgeType::LeftBottom | EdgeType::LeftTop => EdgeType::Left,
            _ => unreachable!("unexpected edge type for corner edge"),
        };
        let side_edge_opt = edges
            .iter()
            .find(|e| e.edge_type == expected_side_edge_type);
        // No side edge found, nothing to draw (should not happen)
        if let Some(side_edge) = side_edge_opt {
            let line_width_f64 = image_params.line_width as f64;
            let line_width_i32 = image_params.line_width as i32;

            // NOTE: Select y_offset of the corner edge based on the cell width.
            // The hard-coded value `height / 10.0` is based on the assumption that the cell
            // has a 1:1 aspect ratio, and does not work well for non-1:1 ratios.
            let y_offset = if image_params.width == image_params.height {
                image_params.height as f64 / 10.0
            } else {
                image_params.height as f64 / 2.0 - image_params.corner_radius() as f64
            };

            match corner_edge.edge_type {
                EdgeType::RightBottom | EdgeType::LeftBottom => {
                    let start_pos_center = Point::new(
                        (side_edge.pos_x * image_params.width as usize) as f64
                            + (image_params.width as f64 / 2.0),
                        image_params.height as f64 / 2.0,
                    );
                    let end_pos_center = Point::new(
                        (corner_edge.pos_x * image_params.width as usize) as f64
                            + (image_params.width as f64 / 2.0),
                        y_offset,
                    );

                    let line_vec = end_pos_center - start_pos_center;
                    let unit_vec = line_vec.normalize();
                    let normal_vec = unit_vec.perpendicular();

                    let line_start =
                        start_pos_center + unit_vec * (image_params.circle_outer_radius as f64);
                    let line_start_1 = line_start + normal_vec * (line_width_f64 / 2.0);
                    let line_start_2 = line_start - normal_vec * (line_width_f64 / 2.0);

                    let half_width = line_width_f64 / 2.0;
                    let slope = unit_vec.y / unit_vec.x;

                    let vertical_left_x = end_pos_center.x - half_width;
                    let vertical_right_x = end_pos_center.x + half_width;

                    let corner_1 = Point::new(
                        vertical_right_x,
                        line_start_1.y + slope * (vertical_right_x - line_start_1.x),
                    );
                    let corner_2 = Point::new(
                        vertical_left_x,
                        line_start_2.y + slope * (vertical_left_x - line_start_2.x),
                    );

                    let vertices = [line_start_1, corner_1, corner_2, line_start_2];

                    let (min_x, min_y, max_x, max_y) = bounding_box_u32(&vertices);
                    for y in min_y..max_y {
                        for x in min_x..max_x {
                            if x < img_buf.width() && y < img_buf.height() {
                                let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);

                                if p.is_inside_polygon(&vertices) {
                                    let pixel = img_buf.get_pixel_mut(x, y);
                                    let color =
                                        image_params.edge_color(side_edge.associated_line_pos_x);
                                    *pixel = color;
                                }
                            }
                        }
                    }

                    let y_end = corner_1.y.max(corner_2.y) as u32;
                    let end_center_x_i32 = end_pos_center.x as i32;
                    let x_start = end_center_x_i32 - line_width_i32 / 2;
                    for y in 0..y_end {
                        for i in 0..line_width_i32 {
                            let x = (x_start + i) as u32;
                            if x < img_buf.width() && y < img_buf.height() {
                                let pixel = img_buf.get_pixel_mut(x, y);
                                let color =
                                    image_params.edge_color(side_edge.associated_line_pos_x);
                                *pixel = color;
                            }
                        }
                    }
                }
                EdgeType::RightTop | EdgeType::LeftTop => {
                    let start_pos_center = Point::new(
                        (side_edge.pos_x * image_params.width as usize) as f64
                            + (image_params.width as f64 / 2.0),
                        image_params.height as f64 / 2.0,
                    );
                    let end_pos_center = Point::new(
                        (corner_edge.pos_x * image_params.width as usize) as f64
                            + (image_params.width as f64 / 2.0),
                        image_params.height as f64 - y_offset,
                    );

                    let line_vec = end_pos_center - start_pos_center;
                    let unit_vec = line_vec.normalize();
                    let normal_vec = unit_vec.perpendicular();

                    let line_start =
                        start_pos_center + unit_vec * (image_params.circle_outer_radius as f64);
                    let line_start_1 = line_start + normal_vec * (line_width_f64 / 2.0);
                    let line_start_2 = line_start - normal_vec * (line_width_f64 / 2.0);

                    let half_width = line_width_f64 / 2.0;
                    let slope = unit_vec.y / unit_vec.x;

                    let vertical_left_x = end_pos_center.x - half_width;
                    let vertical_right_x = end_pos_center.x + half_width;

                    let corner_1 = Point::new(
                        vertical_left_x,
                        line_start_1.y + slope * (vertical_left_x - line_start_1.x),
                    );
                    let corner_2 = Point::new(
                        vertical_right_x,
                        line_start_2.y + slope * (vertical_right_x - line_start_2.x),
                    );

                    let vertices = [line_start_1, corner_1, corner_2, line_start_2];

                    let (min_x, min_y, max_x, max_y) = bounding_box_u32(&vertices);
                    for y in min_y..max_y {
                        for x in min_x..max_x {
                            if x < img_buf.width() && y < img_buf.height() {
                                let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);

                                if p.is_inside_polygon(&vertices) {
                                    let pixel = img_buf.get_pixel_mut(x, y);
                                    let color =
                                        image_params.edge_color(side_edge.associated_line_pos_x);
                                    *pixel = color;
                                }
                            }
                        }
                    }

                    let y_start = corner_1.y.min(corner_2.y) as u32;
                    let end_center_x_i32 = end_pos_center.x as i32;
                    let x_start = end_center_x_i32 - line_width_i32 / 2;
                    for y in (y_start + 1)..image_params.height as u32 {
                        for i in 0..line_width_i32 {
                            let x = (x_start + i) as u32;
                            if x < img_buf.width() && y < img_buf.height() {
                                let pixel = img_buf.get_pixel_mut(x, y);
                                let color =
                                    image_params.edge_color(side_edge.associated_line_pos_x);
                                *pixel = color;
                            }
                        }
                    }
                }
                _ => unreachable!("unexpected edge type for corner edge"),
            }
        }
    }
}

fn build_image(img_buf: &[u8], image_width: u32, image_height: u32) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    image::write_buffer_with_format(
        &mut bytes,
        img_buf,
        image_width,
        image_height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .unwrap();
    bytes.into_inner()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use image::GenericImage;
    use rstest::rstest;

    use crate::config::GraphColorConfig;

    use super::*;
    use EdgeType::*;

    const OUTPUT_DIR: &str = "./out/ut/graph/image";

    type TestParam = (usize, Vec<(EdgeType, usize, usize)>);

    // Note: The output contents are not verified by the code.

    #[rstest]
    #[case("default_params_rounded", GraphStyle::Rounded)]
    #[case("default_params_angular", GraphStyle::Angular)]
    fn test_calc_graph_row_image_default_params(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = simple_test_params();
        let cell_count = 4;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let image_params = ImageParams::new(&graph_color_set, cell_width_type);
        let drawing_pixels = DrawingPixels::new(&image_params);

        test_calc_graph_row_image(
            params,
            cell_count,
            image_params,
            drawing_pixels,
            graph_style,
            file_name,
        );
    }

    #[rstest]
    #[case("wide_image_rounded", GraphStyle::Rounded)]
    #[case("wide_image_angular", GraphStyle::Angular)]
    fn test_calc_graph_row_image_wide_image(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = simple_test_params();
        let cell_count = 4;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let mut image_params = ImageParams::new(&graph_color_set, cell_width_type);
        image_params.width = 100;
        let drawing_pixels = DrawingPixels::new(&image_params);

        test_calc_graph_row_image(
            params,
            cell_count,
            image_params,
            drawing_pixels,
            graph_style,
            file_name,
        );
    }

    #[rstest]
    #[case("tall_image_rounded", GraphStyle::Rounded)]
    #[case("tall_image_angular", GraphStyle::Angular)]
    fn test_calc_graph_row_image_tall_image(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = simple_test_params();
        let cell_count = 4;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let mut image_params = ImageParams::new(&graph_color_set, cell_width_type);
        image_params.height = 100;
        let drawing_pixels = DrawingPixels::new(&image_params);

        test_calc_graph_row_image(
            params,
            cell_count,
            image_params,
            drawing_pixels,
            graph_style,
            file_name,
        );
    }

    #[rstest]
    #[case("single_cell_width_rounded", GraphStyle::Rounded)]
    #[case("single_cell_width_angular", GraphStyle::Angular)]
    fn test_calc_graph_row_image_single_cell_width(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = simple_test_params();
        let cell_count = 4;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Single;
        let image_params = ImageParams::new(&graph_color_set, cell_width_type);
        let drawing_pixels = DrawingPixels::new(&image_params);

        test_calc_graph_row_image(
            params,
            cell_count,
            image_params,
            drawing_pixels,
            graph_style,
            file_name,
        );
    }

    #[rstest]
    #[case("circle_radius_rounded", GraphStyle::Rounded)]
    #[case("circle_radius_angular", GraphStyle::Angular)]
    fn test_calc_graph_row_image_circle_radius(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = straight_test_params();
        let cell_count = 2;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let mut image_params = ImageParams::new(&graph_color_set, cell_width_type);
        image_params.circle_inner_radius = 5;
        image_params.circle_outer_radius = 12;
        let drawing_pixels = DrawingPixels::new(&image_params);

        test_calc_graph_row_image(
            params,
            cell_count,
            image_params,
            drawing_pixels,
            graph_style,
            file_name,
        );
    }

    #[rstest]
    #[case("line_width_rounded", GraphStyle::Rounded)]
    #[case("line_width_angular", GraphStyle::Angular)]
    fn test_calc_graph_row_image_line_width(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = straight_test_params();
        let cell_count = 2;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let mut image_params = ImageParams::new(&graph_color_set, cell_width_type);
        image_params.line_width = 1;
        let drawing_pixels = DrawingPixels::new(&image_params);

        test_calc_graph_row_image(
            params,
            cell_count,
            image_params,
            drawing_pixels,
            graph_style,
            file_name,
        );
    }

    #[rstest]
    #[case("color_rounded", GraphStyle::Rounded)]
    #[case("color_angular", GraphStyle::Angular)]
    fn test_calc_graph_row_image_color(#[case] file_name: &str, #[case] graph_style: GraphStyle) {
        let params = branches_test_params();
        let cell_count = 7;
        let graph_color_config = GraphColorConfig {
            branches: vec![
                "#c8c864".into(),
                "#64c8c8".into(),
                "#646464".into(),
                "#c864c8".into(),
            ],
            edge: "#ffffff".into(),
            background: "#00ff0070".into(),
        };
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let image_params = ImageParams::new(&graph_color_set, cell_width_type);
        let drawing_pixels = DrawingPixels::new(&image_params);

        test_calc_graph_row_image(
            params,
            cell_count,
            image_params,
            drawing_pixels,
            graph_style,
            file_name,
        );
    }

    #[rustfmt::skip]
    fn simple_test_params() -> Vec<TestParam> {
        vec![
            (1, vec![(LeftBottom, 0, 0), (Left, 1, 0), (Down, 1, 1), (Right, 1, 3), (Horizontal, 2, 3), (RightBottom, 3, 3)]),
            (3, vec![(Vertical, 0, 0), (Up, 3, 3), (Down, 3, 3)]),
            (2, vec![(LeftTop, 0, 0), (Horizontal, 1, 0), (Left, 2, 0), (Up, 2, 2), (Right, 2, 3), (RightTop, 3, 3)]),
        ]
    }

    #[rustfmt::skip]
    fn straight_test_params() -> Vec<TestParam> {
        vec![
            (0, vec![(Up, 0, 0), (Down, 0, 0)]),
            (0, vec![(Up, 0, 0), (Down, 0, 0), (Right, 0, 1), (RightBottom, 1, 1)]),
            (1, vec![(Vertical, 0, 0), (Up, 1, 1), (Down, 1, 1)]),
            (0, vec![(Up, 0, 0), (Down, 0, 0), (Right, 0, 1), (RightTop, 1, 1)]),
        ]
    }

    #[rustfmt::skip]
    fn branches_test_params() -> Vec<TestParam> {
        vec![
            (0, vec![(Up, 0, 0), (Down, 0, 0),
                    (Right, 0, 1), (RightBottom, 1, 1),
                    (Right, 0, 2), (Horizontal, 1, 2), (RightBottom, 2, 2),
                    (Right, 0, 3), (Horizontal, 1, 3), (Horizontal, 2, 3), (RightBottom, 3, 3),
                    (Right, 0, 4), (Horizontal, 1, 4), (Horizontal, 2, 4), (Horizontal, 3, 4), (RightBottom, 4, 4),
                    (Right, 0, 5), (Horizontal, 1, 5), (Horizontal, 2, 5), (Horizontal, 3, 5), (Horizontal, 4, 5), (RightBottom, 5, 5),
                    (Right, 0, 6), (Horizontal, 1, 6), (Horizontal, 2, 6), (Horizontal, 3, 6), (Horizontal, 4, 6), (Horizontal, 5, 6), (RightBottom, 6, 6)]),
            (6, vec![(Vertical, 0, 0), (Vertical, 1, 1), (Vertical, 2, 2), (Vertical, 3, 3), (Vertical, 4, 4), (Vertical, 5, 5), (Down, 6, 6), (Up, 6, 6)]),
        ]
    }

    fn test_calc_graph_row_image(
        params: Vec<TestParam>,
        cell_count: usize,
        image_params: ImageParams,
        drawing_pixels: DrawingPixels,
        graph_style: GraphStyle,
        file_name: &str,
    ) {
        let graph_row_images: Vec<GraphRowImage> = params
            .into_iter()
            .map(|(commit_pos_x, edges)| {
                let edges: Vec<Edge> = edges
                    .into_iter()
                    .map(|t| Edge::new(t.0, t.1, t.2))
                    .collect();
                calc_graph_row_image(
                    commit_pos_x,
                    cell_count,
                    &edges,
                    &image_params,
                    &drawing_pixels,
                    graph_style,
                )
            })
            .collect();

        save_image(&graph_row_images, &image_params, cell_count, file_name);
    }

    fn save_image(
        graph_row_images: &[GraphRowImage],
        image_params: &ImageParams,
        cell_count: usize,
        file_name: &str,
    ) {
        let rows_len = graph_row_images.len() as u32;
        let image_width = image_params.width as u32 * cell_count as u32;
        let image_height = image_params.height as u32 * rows_len;

        let mut img_buf: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            image::ImageBuffer::new(image_width, image_height);

        for (i, graph_row_image) in graph_row_images.iter().enumerate() {
            let image = image::load_from_memory(&graph_row_image.bytes).unwrap();
            let y = image_params.height as u32 * (rows_len - (i as u32) - 1);
            img_buf.copy_from(&image, 0, y).unwrap();

            for x in 0..cell_count {
                let x_offset = x as u32 * image_params.width as u32;
                let y_offset = y;
                draw_border(&mut img_buf, image_params, x_offset, y_offset);
            }
        }

        create_output_dirs(OUTPUT_DIR);
        let file_name = format!("{OUTPUT_DIR}/{file_name}.png");
        image::save_buffer(
            file_name,
            &img_buf,
            image_width,
            image_height,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }

    fn draw_border(
        img_buf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        image_params: &ImageParams,
        x_offset: u32,
        y_offset: u32,
    ) {
        for x in 0..image_params.width {
            for y in 0..image_params.height {
                if x == 0 || x == image_params.width - 1 || y == 0 || y == image_params.height - 1 {
                    img_buf.put_pixel(
                        x as u32 + x_offset,
                        y as u32 + y_offset,
                        image::Rgba([255, 0, 0, 50]),
                    );
                }
            }
        }
    }

    fn create_output_dirs(path: &str) {
        let path = Path::new(path);
        std::fs::create_dir_all(path).unwrap();
    }
}
