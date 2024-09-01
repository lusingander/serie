use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Formatter},
    io::Cursor,
};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    color::ColorSet,
    git::CommitHash,
    graph::{
        cache::{ImageCache, ImageCacheDirKey, ImageCacheFileKey},
        Edge, EdgeType, Graph,
    },
    protocol::ImageProtocol,
};

#[derive(Debug)]
pub struct GraphImageManager<'a> {
    encoded_image_map: HashMap<CommitHash, String>,

    graph: &'a Graph<'a>,
    options: GraphImageOptions,
    image_protocol: ImageProtocol,
}

impl<'a> GraphImageManager<'a> {
    pub fn new(
        graph: &'a Graph,
        options: GraphImageOptions,
        image_protocol: ImageProtocol,
        preload: bool,
    ) -> Self {
        let encoded_image_map = if preload {
            let graph_image = build_graph_image(graph, &options);
            graph
                .commits
                .iter()
                .enumerate()
                .map(|(i, commit)| {
                    let edges = &graph.edges[i];
                    let graph_row_image = &graph_image.images[edges];
                    let image_cell_width = graph_row_image.cell_count * 2;
                    let image = image_protocol.encode(&graph_row_image.bytes, image_cell_width);
                    (commit.commit_hash.clone(), image)
                })
                .collect()
        } else {
            HashMap::new()
        };

        Self {
            encoded_image_map,
            graph,
            options,
            image_protocol,
        }
    }

    pub fn encoded_image(&self, commit_hash: &CommitHash) -> &str {
        self.encoded_image_map.get(commit_hash).unwrap()
    }

    pub fn load_encoded_image(&mut self, commit_hash: &CommitHash) {
        if self.encoded_image_map.contains_key(commit_hash) {
            return;
        }
        let graph_row_image = build_single_graph_row_image(self.graph, &self.options, commit_hash);
        let image_cell_width = graph_row_image.cell_count * 2;
        let image = self
            .image_protocol
            .encode(&graph_row_image.bytes, image_cell_width);
        self.encoded_image_map.insert(commit_hash.clone(), image);
    }
}

#[derive(Debug, Default)]
pub struct GraphImage {
    pub images: HashMap<Vec<Edge>, GraphRowImage>,
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

struct ImageParams {
    width: u16,
    height: u16,
    line_width: u16,
    circle_inner_radius: u16,
    circle_outer_radius: u16,
    edge_colors: Vec<image::Rgba<u8>>,
}

impl ImageParams {
    fn new(color_set: &ColorSet) -> Self {
        let width = 50;
        let height = 50;
        let line_width = 5;
        let circle_inner_radius = 10;
        let circle_outer_radius = 14;
        let edge_colors = color_set
            .colors
            .iter()
            .map(|c| c.to_image_color())
            .collect();
        Self {
            width,
            height,
            line_width,
            circle_inner_radius,
            circle_outer_radius,
            edge_colors,
        }
    }

    fn edge_color(&self, index: usize) -> image::Rgba<u8> {
        self.edge_colors[index % self.edge_colors.len()]
    }
}

#[derive(Debug, Clone)]
pub struct GraphImageOptions {
    color_set: ColorSet,
    no_cache: bool,
}

impl GraphImageOptions {
    pub fn new(color_set: ColorSet, no_cache: bool) -> Self {
        Self {
            color_set,
            no_cache,
        }
    }
}

fn build_single_graph_row_image(
    graph: &Graph<'_>,
    options: &GraphImageOptions,
    commit_hash: &CommitHash,
) -> GraphRowImage {
    let image_params = ImageParams::new(&options.color_set);
    let image_cache = setup_image_cache(&image_params, options);

    let drawing_pixels = DrawingPixels::new(&image_params);

    let (pos_x, pos_y) = graph.commit_pos_map[&commit_hash];
    let edges = &graph.edges[pos_y];

    let cell_count = graph.max_pos_x + 1;

    if let Some(image_cache) = &image_cache {
        let image_cache_file_key = ImageCacheFileKey::new(pos_x, cell_count, edges.clone());
        image_cache
            .load_image_cache(&image_cache_file_key)
            .unwrap_or_else(|| {
                let graph_row_image =
                    calc_graph_row_image(pos_x, cell_count, edges, &image_params, &drawing_pixels);
                image_cache.save_image_cache(&image_cache_file_key, &graph_row_image);
                graph_row_image
            })
    } else {
        calc_graph_row_image(pos_x, cell_count, edges, &image_params, &drawing_pixels)
    }
}

pub fn build_graph_image(graph: &Graph<'_>, options: &GraphImageOptions) -> GraphImage {
    let image_params = ImageParams::new(&options.color_set);
    let image_cache = setup_image_cache(&image_params, options);

    let drawing_pixels = DrawingPixels::new(&image_params);

    let graph_row_sources: HashSet<(usize, &Vec<Edge>)> = graph
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
            let graph_row_image = if let Some(image_cache) = &image_cache {
                let image_cache_file_key = ImageCacheFileKey::new(pos_x, cell_count, edges.clone());
                image_cache
                    .load_image_cache(&image_cache_file_key)
                    .unwrap_or_else(|| {
                        let graph_row_image = calc_graph_row_image(
                            pos_x,
                            cell_count,
                            edges,
                            &image_params,
                            &drawing_pixels,
                        );
                        image_cache.save_image_cache(&image_cache_file_key, &graph_row_image);
                        graph_row_image
                    })
            } else {
                calc_graph_row_image(pos_x, cell_count, edges, &image_params, &drawing_pixels)
            };
            (edges.clone(), graph_row_image)
        })
        .collect();

    GraphImage { images }
}

fn setup_image_cache(
    image_params: &ImageParams,
    options: &GraphImageOptions,
) -> Option<ImageCache> {
    if options.no_cache {
        None
    } else {
        let image_cache_dir_key = ImageCacheDirKey::new(
            image_params.width,
            image_params.height,
            image_params.line_width,
            image_params.circle_inner_radius,
            image_params.circle_outer_radius,
            image_params.edge_colors.clone(),
        );
        Some(ImageCache::new(image_cache_dir_key))
    }
}

type Pixels = HashSet<(i32, i32)>;

struct DrawingPixels {
    circle: Pixels,
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
    fn new(image_params: &ImageParams) -> Self {
        let circle = calc_commit_circle_drawing_pixels(image_params);
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
    // Bresenham's circle algorithm
    let center_x = (image_params.width / 2) as i32;
    let center_y = (image_params.height / 2) as i32;
    let radius = image_params.circle_inner_radius as i32;

    let mut x = radius;
    let mut y = 0;
    let mut p = 1 - radius;

    let mut pixels = Pixels::new();

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
    let half_line_width = (image_params.line_width as i32) / 2;

    let mut pixels = Pixels::new();
    for y in 0..image_params.height {
        for x in (center_x - half_line_width)..=(center_x + half_line_width) {
            pixels.insert((x, y as i32));
        }
    }
    pixels
}

fn calc_horizontal_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_y = (image_params.height / 2) as i32;
    let half_line_width = (image_params.line_width as i32) / 2;

    let mut pixels = Pixels::new();
    for y in (center_y - half_line_width)..=(center_y + half_line_width) {
        for x in 0..image_params.width {
            pixels.insert((x as i32, y));
        }
    }
    pixels
}

fn calc_up_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_x = (image_params.width / 2) as i32;
    let half_line_width = (image_params.line_width as i32) / 2;
    let circle_center_y = (image_params.height / 2) as i32;
    let circle_outer_radius = image_params.circle_outer_radius as i32;

    let mut pixels = Pixels::new();
    for y in 0..=(circle_center_y - circle_outer_radius) {
        for x in (center_x - half_line_width)..=(center_x + half_line_width) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_down_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_x = (image_params.width / 2) as i32;
    let half_line_width = (image_params.line_width as i32) / 2;
    let circle_center_y = (image_params.height / 2) as i32;
    let circle_outer_radius = image_params.circle_outer_radius as i32;

    let mut pixels = Pixels::new();
    for y in (circle_center_y + circle_outer_radius)..(image_params.height as i32) {
        for x in (center_x - half_line_width)..=(center_x + half_line_width) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_left_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_y = (image_params.height / 2) as i32;
    let half_line_width = (image_params.line_width as i32) / 2;
    let circle_center_x = (image_params.width / 2) as i32;
    let circle_outer_radius = image_params.circle_outer_radius as i32;

    let mut pixels = Pixels::new();
    for y in (center_y - half_line_width)..=(center_y + half_line_width) {
        for x in 0..=(circle_center_x - circle_outer_radius) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_right_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    let center_y = (image_params.height / 2) as i32;
    let half_line_width = (image_params.line_width as i32) / 2;
    let circle_center_x = (image_params.width / 2) as i32;
    let circle_outer_radius = image_params.circle_outer_radius as i32;

    let mut pixels = Pixels::new();
    for y in (center_y - half_line_width)..=(center_y + half_line_width) {
        for x in (circle_center_x + circle_outer_radius)..=(image_params.width as i32) {
            pixels.insert((x, y));
        }
    }
    pixels
}

fn calc_right_top_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    calc_corner_edge_drawing_pixels(image_params, 0, image_params.height)
}

fn calc_left_top_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    calc_corner_edge_drawing_pixels(image_params, image_params.width, image_params.height)
}

fn calc_right_bottom_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    calc_corner_edge_drawing_pixels(image_params, 0, 0)
}

fn calc_left_bottom_edge_drawing_pixels(image_params: &ImageParams) -> Pixels {
    calc_corner_edge_drawing_pixels(image_params, image_params.width, 0)
}

fn calc_corner_edge_drawing_pixels(
    image_params: &ImageParams,
    curve_center_x: u16,
    curve_center_y: u16,
) -> Pixels {
    // Bresenham's circle algorithm
    let curve_center_x = curve_center_x as i32;
    let curve_center_y = curve_center_y as i32;
    let half_line_width = (image_params.line_width as i32) / 2;
    let adjust = if image_params.line_width % 2 == 0 {
        0
    } else {
        1
    };
    let inner_radius = (image_params.width / 2) as i32 - half_line_width - adjust;
    let outer_radius = (image_params.width / 2) as i32 + half_line_width;

    let mut x = inner_radius;
    let mut y = 0;
    let mut p = 1 - inner_radius;

    let mut inner_pixels = Pixels::new();

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

    let mut outer_pixels = Pixels::new();

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

    outer_pixels
        .difference(&inner_pixels)
        .filter(|p| {
            p.0 >= 0
                && p.0 < image_params.width as i32
                && p.1 >= 0
                && p.1 < image_params.height as i32
        })
        .cloned()
        .collect()
}

fn calc_graph_row_image(
    commit_pos_x: usize,
    cell_count: usize,
    edges: &[Edge],
    image_params: &ImageParams,
    drawing_pixels: &DrawingPixels,
) -> GraphRowImage {
    let image_width = (image_params.width as usize * cell_count) as u32;
    let image_height = image_params.height as u32;

    let mut img_buf = image::ImageBuffer::new(image_width, image_height);

    draw_commit_circle(&mut img_buf, commit_pos_x, image_params, drawing_pixels);

    for edge in edges {
        draw_edge(&mut img_buf, edge, image_params, drawing_pixels)
    }

    let bytes = build_image(&img_buf, image_width, image_height);

    GraphRowImage { bytes, cell_count }
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
