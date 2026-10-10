use std::{
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    io::Cursor,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    color::GraphColorSet,
    git::CommitHash,
    graph::{
        geometry::{bounding_box_u32, Point},
        Edge, EdgeType, Graph,
    },
    protocol::{ImageProtocol, PreparedImage},
};

mod antialias;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphStyle {
    Rounded,
    Angular,
    Curved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphImageWidthMode {
    Compact,
    Fixed,
}

#[derive(Debug)]
pub struct GraphImageManager<'a> {
    prepared_image_map: FxHashMap<CommitHash, PreparedImage>,
    image_ids: FxHashSet<u32>,
    pending_uploads: Vec<String>,

    graph: &'a Graph<'a>,
    cell_width_type: CellWidthType,
    graph_style: GraphStyle,
    image_width_mode: GraphImageWidthMode,
    image_params: ImageParams,
    drawing_pixels: DrawingPixels,
    image_protocol: ImageProtocol,
    session_nonce: u32,
}

impl<'a> GraphImageManager<'a> {
    pub fn new(
        graph: &'a Graph,
        graph_color_set: &GraphColorSet,
        cell_width_type: CellWidthType,
        graph_style: GraphStyle,
        antialias: bool,
        image_width_mode: GraphImageWidthMode,
        image_protocol: ImageProtocol,
    ) -> Self {
        let image_params = ImageParams::new(graph_color_set, cell_width_type, antialias);
        let drawing_pixels = DrawingPixels::new(&image_params);

        GraphImageManager {
            prepared_image_map: FxHashMap::default(),
            image_ids: FxHashSet::default(),
            pending_uploads: Vec::default(),
            graph,
            cell_width_type,
            graph_style,
            image_width_mode,
            image_params,
            drawing_pixels,
            image_protocol,
            session_nonce: create_session_nonce(),
        }
    }

    pub fn prepared_image(&self, commit_hash: &CommitHash) -> &PreparedImage {
        self.prepared_image_map.get(commit_hash).unwrap()
    }

    pub fn graph(&self) -> &'a Graph<'a> {
        self.graph
    }

    pub fn graph_cell_width(&self) -> u16 {
        let cell_count = (self.graph.max_pos_x + 1) as u16;
        match self.cell_width_type {
            CellWidthType::Double => cell_count * 2,
            CellWidthType::Single => cell_count,
        }
    }

    pub fn image_ids(&self) -> &FxHashSet<u32> {
        &self.image_ids
    }

    pub fn drain_pending_uploads(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_uploads)
    }

    pub fn ensure_uploaded(&mut self, commit_hash: &CommitHash) {
        if self.prepared_image_map.contains_key(commit_hash) {
            return;
        }
        let image_id = graph_image_id(self.session_nonce, commit_hash);
        let graph_row_image = build_single_graph_row_image(
            self.graph,
            &self.image_params,
            &self.drawing_pixels,
            self.graph_style,
            self.image_width_mode,
            commit_hash,
        );
        let mut image =
            graph_row_image.prepare(self.cell_width_type, self.image_protocol, image_id);
        if let Some(upload_data) = image.take_upload_data() {
            self.pending_uploads.push(upload_data);
        }
        self.prepared_image_map.insert(commit_hash.clone(), image);
        self.image_ids.insert(image_id);
    }
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
    fn prepare(
        &self,
        cell_width_type: CellWidthType,
        image_protocol: ImageProtocol,
        image_id: u32,
    ) -> PreparedImage {
        let image_cell_width = match cell_width_type {
            CellWidthType::Double => self.cell_count * 2,
            CellWidthType::Single => self.cell_count,
        };
        image_protocol.prepare_image(&self.bytes, image_cell_width, image_id)
    }
}

fn create_session_nonce() -> u32 {
    let mut hasher = rustc_hash::FxHasher::default();
    process::id().hash(&mut hasher);
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    hasher.finish() as u32
}

fn graph_image_id(session_nonce: u32, commit_hash: &CommitHash) -> u32 {
    let mut hasher = rustc_hash::FxHasher::default();
    session_nonce.hash(&mut hasher);
    commit_hash.hash(&mut hasher);
    hasher.finish() as u32
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
    antialias: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellWidthType {
    Double, // 2 cells
    Single,
}

impl ImageParams {
    pub fn new(
        graph_color_set: &GraphColorSet,
        cell_width_type: CellWidthType,
        antialias: bool,
    ) -> Self {
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
            antialias,
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
    image_width_mode: GraphImageWidthMode,
    commit_hash: &CommitHash,
) -> GraphRowImage {
    let (pos_x, pos_y) = graph.commit_pos_map[&commit_hash];
    let edges = &graph.edges[pos_y];

    let max_pos_x = match image_width_mode {
        GraphImageWidthMode::Compact => edges.iter().map(|e| e.pos_x).fold(pos_x, usize::max),
        GraphImageWidthMode::Fixed => graph.max_pos_x,
    };

    let cell_count = max_pos_x + 1;

    calc_graph_row_image(
        pos_x,
        cell_count,
        edges,
        image_params,
        drawing_pixels,
        graph_style,
    )
}

type Pixels = FxHashSet<(i32, i32)>;

#[derive(Debug)]
pub struct DrawingPixels {
    antialias: Option<antialias::Masks>,
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
            antialias: image_params
                .antialias
                .then(|| antialias::Masks::new(image_params)),
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

pub fn calc_graph_row_image(
    commit_pos_x: usize,
    cell_count: usize,
    edges: &[Edge],
    image_params: &ImageParams,
    drawing_pixels: &DrawingPixels,
    graph_style: GraphStyle,
) -> GraphRowImage {
    let image_width = (image_params.width as usize * cell_count) as u32;
    let image_height = image_params.height as u32;

    if image_params.antialias {
        let image = antialias::render(
            commit_pos_x,
            cell_count,
            edges,
            image_params,
            drawing_pixels,
            graph_style,
        );
        return GraphRowImage {
            bytes: build_image(&image, image_width, image_height),
            cell_count,
        };
    }

    let mut img_buf = image::ImageBuffer::new(image_width, image_height);

    draw_background(&mut img_buf, image_params);
    draw_commit_circle(&mut img_buf, commit_pos_x, image_params, drawing_pixels);

    match graph_style {
        GraphStyle::Curved => {
            for edge in edges.iter().filter(|e| e.edge_type.is_vertically_related()) {
                draw_edge(&mut img_buf, edge, image_params, drawing_pixels);
            }
            draw_curved_connected_edge(&mut img_buf, edges, image_params);
        }
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
    for corner in edges
        .iter()
        .filter(|e| !e.edge_type.is_vertically_related())
    {
        let side_type = match corner.edge_type {
            EdgeType::RightBottom | EdgeType::RightTop => EdgeType::Right,
            EdgeType::LeftBottom | EdgeType::LeftTop => EdgeType::Left,
            _ => continue,
        };
        let Some(side) = edges.iter().find(|e| e.edge_type == side_type) else {
            continue;
        };
        let (vertices, ys, x_start) = diagonal_connection(side, corner, image_params);
        let color = image_params.edge_color(side.associated_line_pos_x);
        let (min_x, min_y, max_x, max_y) = bounding_box_u32(&vertices);
        for y in min_y..max_y.min(img_buf.height()) {
            for x in min_x..max_x.min(img_buf.width()) {
                let p = Point::new(x as f64 + 0.5, y as f64 + 0.5);
                if p.is_inside_polygon(&vertices) {
                    *img_buf.get_pixel_mut(x, y) = color;
                }
            }
        }
        for y in ys {
            for i in 0..i32::from(image_params.line_width) {
                let x = (x_start + i) as u32;
                if x < img_buf.width() && y < img_buf.height() {
                    *img_buf.get_pixel_mut(x, y) = color;
                }
            }
        }
    }
}

fn diagonal_connection(
    side: &Edge,
    corner: &Edge,
    image_params: &ImageParams,
) -> ([Point; 4], std::ops::Range<u32>, i32) {
    let half_width = f64::from(image_params.line_width) / 2.0;
    // Keep the existing diagonal-to-vertical junction for each cell aspect ratio.
    let y_offset = if image_params.width == image_params.height {
        f64::from(image_params.height) / 10.0
    } else {
        f64::from(image_params.height) / 2.0 - f64::from(image_params.corner_radius())
    };
    let turns_up = matches!(
        corner.edge_type,
        EdgeType::RightBottom | EdgeType::LeftBottom
    );
    let start = Point::new(
        (side.pos_x * image_params.width as usize) as f64 + f64::from(image_params.width) / 2.0,
        f64::from(image_params.height) / 2.0,
    );
    let end = Point::new(
        (corner.pos_x * image_params.width as usize) as f64 + f64::from(image_params.width) / 2.0,
        if turns_up {
            y_offset
        } else {
            f64::from(image_params.height) - y_offset
        },
    );
    let unit = (end - start).normalize();
    let normal = unit.perpendicular();
    let line_start = start + unit * f64::from(image_params.circle_outer_radius);
    let line_start_1 = line_start + normal * half_width;
    let line_start_2 = line_start - normal * half_width;
    let slope = unit.y / unit.x;
    let (x1, x2) = if turns_up {
        (end.x + half_width, end.x - half_width)
    } else {
        (end.x - half_width, end.x + half_width)
    };
    let corner_1 = Point::new(x1, line_start_1.y + slope * (x1 - line_start_1.x));
    let corner_2 = Point::new(x2, line_start_2.y + slope * (x2 - line_start_2.x));
    let ys = if turns_up {
        0..corner_1.y.max(corner_2.y) as u32
    } else {
        (corner_1.y.min(corner_2.y) as u32 + 1)..u32::from(image_params.height)
    };
    let x_start = end.x as i32 - i32::from(image_params.line_width) / 2;
    (
        [line_start_1, corner_1, corner_2, line_start_2],
        ys,
        x_start,
    )
}

fn draw_curved_connected_edge(
    img_buf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    edges: &[Edge],
    image_params: &ImageParams,
) {
    let corner_edges = edges.iter().filter(|e| {
        matches!(
            e.edge_type,
            EdgeType::RightBottom | EdgeType::LeftBottom | EdgeType::RightTop | EdgeType::LeftTop
        )
    });

    for corner in corner_edges {
        let side_type = match corner.edge_type {
            EdgeType::RightBottom | EdgeType::RightTop => EdgeType::Right,
            _ => EdgeType::Left,
        };
        let Some(side) = edges.iter().find(|e| {
            e.associated_line_pos_x == corner.associated_line_pos_x && e.edge_type == side_type
        }) else {
            continue;
        };

        let points = curved_connection_points(side, corner, image_params);
        let color = image_params.edge_color(corner.associated_line_pos_x);
        draw_bezier_curve(img_buf, points, image_params.line_width, color);
    }
}

fn curved_connection_points(side: &Edge, corner: &Edge, image_params: &ImageParams) -> [Point; 4] {
    let direction = if side.edge_type == EdgeType::Right {
        1.0
    } else {
        -1.0
    };
    // Align stroke centers with the existing straight-edge pixel masks.
    let pixel_offset = f64::from(image_params.line_width % 2) / 2.0;
    let center_y = f64::from(image_params.height / 2) + pixel_offset;
    let start_x = (side.pos_x * image_params.width as usize) as f64
        + f64::from(image_params.width / 2)
        + 0.5
        + direction * (f64::from(image_params.circle_outer_radius) + 1.0);
    let end_x = (corner.pos_x * image_params.width as usize) as f64
        + f64::from(image_params.width / 2)
        + pixel_offset;
    let end_y = match corner.edge_type {
        EdgeType::RightBottom | EdgeType::LeftBottom => 0.0,
        _ => f64::from(image_params.height),
    };

    // Leave the circle horizontally and meet the row boundary vertically.
    // Control points stay within the connection's existing row and columns.
    [
        Point::new(start_x, center_y),
        Point::new(start_x + (end_x - start_x) * 0.60, center_y),
        Point::new(end_x, end_y + (center_y - end_y) * 0.65),
        Point::new(end_x, end_y),
    ]
}

fn bezier_point(points: &[Point; 4], t: f64) -> Point {
    let u = 1.0 - t;
    Point::new(
        u * u * u * points[0].x
            + 3.0 * u * u * t * points[1].x
            + 3.0 * u * t * t * points[2].x
            + t * t * t * points[3].x,
        u * u * u * points[0].y
            + 3.0 * u * u * t * points[1].y
            + 3.0 * u * t * t * points[2].y
            + t * t * t * points[3].y,
    )
}

fn draw_bezier_curve(
    img_buf: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    points: [Point; 4],
    line_width: u16,
    color: image::Rgba<u8>,
) {
    let radius = f64::from(line_width) / 2.0;
    let direction = (points[3].x - points[0].x).signum();
    let steps = (((points[3].x - points[0].x).abs() + (points[3].y - points[0].y).abs()) * 3.0)
        .ceil() as usize;
    let boundary_y = if points[3].y == 0.0 {
        0
    } else {
        img_buf.height() - 1
    };
    // Approximate the curve with short segments and fill pixels within the stroke.
    let steps = steps.max(32);
    let mut start = points[0];
    for step in 1..=steps {
        let end = bezier_point(&points, step as f64 / steps as f64);
        let segment = end - start;
        let length_squared = segment.dot(segment);
        let bounds = [
            Point::new(
                start.x.min(end.x) - radius - 1.0,
                start.y.min(end.y) - radius - 1.0,
            ),
            Point::new(
                start.x.max(end.x) + radius + 1.0,
                start.y.max(end.y) + radius + 1.0,
            ),
        ];
        let (min_x, min_y, max_x, max_y) = bounding_box_u32(&bounds);
        for y in min_y..max_y.min(img_buf.height()) {
            if y == boundary_y {
                continue;
            }
            for x in min_x..max_x.min(img_buf.width()) {
                let pixel_center = Point::new(x as f64 + 0.5, y as f64 + 0.5);
                // Keep the same circle-side endpoint, with a flat stroke cap.
                if (pixel_center.x - points[0].x) * direction < 0.0 {
                    continue;
                }
                let t = if length_squared == 0.0 {
                    0.0
                } else {
                    ((pixel_center - start).dot(segment) / length_squared).clamp(0.0, 1.0)
                };
                let distance = (pixel_center - (start + segment * t)).length();
                if distance <= radius {
                    *img_buf.get_pixel_mut(x, y) = color;
                }
            }
        }
        start = end;
    }

    // Match the existing vertical mask exactly at the row boundary.
    let x_start = (points[3].x - radius) as u32;
    for x in x_start..(x_start + u32::from(line_width)).min(img_buf.width()) {
        *img_buf.get_pixel_mut(x, boundary_y) = color;
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
    #[case("default_params_curved", GraphStyle::Curved)]
    fn test_calc_graph_row_image_default_params(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = simple_test_params();
        let cell_count = 4;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let image_params = ImageParams::new(&graph_color_set, cell_width_type, false);
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
    #[case("wide_image_curved", GraphStyle::Curved)]
    fn test_calc_graph_row_image_wide_image(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = simple_test_params();
        let cell_count = 4;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let mut image_params = ImageParams::new(&graph_color_set, cell_width_type, false);
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
    #[case("tall_image_curved", GraphStyle::Curved)]
    fn test_calc_graph_row_image_tall_image(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = simple_test_params();
        let cell_count = 4;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let mut image_params = ImageParams::new(&graph_color_set, cell_width_type, false);
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
    #[case("single_cell_width_curved", GraphStyle::Curved)]
    fn test_calc_graph_row_image_single_cell_width(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = simple_test_params();
        let cell_count = 4;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Single;
        let image_params = ImageParams::new(&graph_color_set, cell_width_type, false);
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
    #[case("circle_radius_curved", GraphStyle::Curved)]
    fn test_calc_graph_row_image_circle_radius(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = straight_test_params();
        let cell_count = 2;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let mut image_params = ImageParams::new(&graph_color_set, cell_width_type, false);
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
    #[case("line_width_curved", GraphStyle::Curved)]
    fn test_calc_graph_row_image_line_width(
        #[case] file_name: &str,
        #[case] graph_style: GraphStyle,
    ) {
        let params = straight_test_params();
        let cell_count = 2;
        let graph_color_config = GraphColorConfig::default();
        let graph_color_set = GraphColorSet::new(&graph_color_config);
        let cell_width_type = CellWidthType::Double;
        let mut image_params = ImageParams::new(&graph_color_set, cell_width_type, false);
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
    #[case("color_curved", GraphStyle::Curved)]
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
        let image_params = ImageParams::new(&graph_color_set, cell_width_type, false);
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
    #[case(CellWidthType::Double)]
    #[case(CellWidthType::Single)]
    fn test_curved_connection_positions(#[case] cell_width_type: CellWidthType) {
        let colors = GraphColorSet::new(&GraphColorConfig::default());
        let params = ImageParams::new(&colors, cell_width_type, false);
        let pixels = DrawingPixels::new(&params);

        for (commit_pos_x, edges) in simple_test_params() {
            let edges: Vec<Edge> = edges
                .into_iter()
                .map(|(t, x, line)| Edge::new(t, x, line))
                .collect();
            let rounded = calc_graph_row_image(
                commit_pos_x,
                4,
                &edges,
                &params,
                &pixels,
                GraphStyle::Rounded,
            );
            let curved = calc_graph_row_image(
                commit_pos_x,
                4,
                &edges,
                &params,
                &pixels,
                GraphStyle::Curved,
            );
            assert_eq!(rounded.cell_count, curved.cell_count);
            let rounded = image::load_from_memory(&rounded.bytes).unwrap().to_rgba8();
            let curved = image::load_from_memory(&curved.bytes).unwrap().to_rgba8();
            assert_eq!(rounded.dimensions(), curved.dimensions());
            assert!(curved.pixels().all(|pixel| matches!(pixel[3], 0 | 255)));

            for y in [0, rounded.height() - 1] {
                for x in 0..rounded.width() {
                    assert_eq!(rounded.get_pixel(x, y), curved.get_pixel(x, y));
                }
            }
            let x_offset = (commit_pos_x * params.width as usize) as u32;
            for &(x, y) in pixels.circle.union(&pixels.circle_edge) {
                let (x, y) = (x as u32 + x_offset, y as u32);
                assert_eq!(rounded.get_pixel(x, y), curved.get_pixel(x, y));
            }
            // Circle-side endpoints stay at the existing left/right positions.
            for edge in edges.iter().filter(|e| matches!(e.edge_type, Left | Right)) {
                let center_x =
                    (edge.pos_x * params.width as usize) as u32 + u32::from(params.width / 2);
                let offset = u32::from(params.circle_outer_radius) + 1;
                let x = if edge.edge_type == Right {
                    center_x + offset
                } else {
                    center_x - offset
                };
                let y = u32::from(params.height / 2);
                assert_eq!(rounded.get_pixel(x, y), curved.get_pixel(x, y));
            }
        }
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
