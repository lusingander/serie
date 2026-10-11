use std::path::Path;

use font8x8::{UnicodeFonts, BASIC_FONTS};
use image::{GenericImage, GenericImageView};

use crate::{color, config, git, graph};

const OUTPUT_DIR: &str = "./out/graph";
const SNAPSHOT_DIR: &str = "./tests/graph";
// Set this environment variable when Git repository output is needed for graph test debugging.
const DUMP_GRAPH_TEST_REPOS_ENV: &str = "SERIE_TEST_DUMP_GRAPH_REPOS";

pub struct GenerateGraphOption {
    output_name: &'static str,
    sort: git::SortCommit,
    style: graph::GraphStyle,
    antialias: bool,
    cell_width_type: graph::CellWidthType,
    max_count: Option<usize>,
    primary_branch: Option<String>,
}

impl GenerateGraphOption {
    pub fn new(
        output_name: &'static str,
        sort: git::SortCommit,
        style: graph::GraphStyle,
    ) -> GenerateGraphOption {
        GenerateGraphOption {
            output_name,
            sort,
            style,
            antialias: false,
            cell_width_type: graph::CellWidthType::Double,
            max_count: None,
            primary_branch: None,
        }
    }

    pub fn with_antialias(mut self) -> GenerateGraphOption {
        self.antialias = true;
        self
    }

    pub fn with_cell_width_type(mut self, cell_width_type: graph::CellWidthType) -> Self {
        self.cell_width_type = cell_width_type;
        self
    }

    pub fn with_max_count(mut self, max_count: usize) -> GenerateGraphOption {
        self.max_count = Some(max_count);
        self
    }

    pub fn with_primary_branch(mut self, primary_branch: impl Into<String>) -> GenerateGraphOption {
        self.primary_branch = Some(primary_branch.into());
        self
    }
}

pub fn assert_graph_snapshots(repo_path: &Path, options: &[GenerateGraphOption]) {
    create_output_dirs(OUTPUT_DIR);
    let errors: Vec<_> = options
        .iter()
        .filter_map(|option| {
            let actual_img = render_graph_image(repo_path, option);
            let actual_file = format!("{}/{}.png", OUTPUT_DIR, option.output_name);
            actual_img.save(actual_file).unwrap();
            compare_graph_image(option, &actual_img).err()
        })
        .collect();
    if !errors.is_empty() {
        panic!("{}", errors.join("\n"));
    }
}

fn render_graph_image(path: &Path, option: &GenerateGraphOption) -> image::RgbaImage {
    // Build graphs in the same way as application
    let max_count = option.max_count;
    let graph_color_config = config::GraphColorConfig::default();
    let graph_color_set = color::GraphColorSet::new(&graph_color_config);
    let cell_width_type = option.cell_width_type;
    let repository = git::Repository::load(path, option.sort, max_count, true).unwrap();
    let graph = graph::calc_graph(&repository, option.primary_branch.as_deref());
    let image_params = graph::ImageParams::new(&graph_color_set, cell_width_type, option.antialias);
    let mut drawing_pixels = graph::DrawingPixels::new(&image_params);
    let row_images: Vec<_> = graph
        .commits
        .iter()
        .map(|commit| {
            let graph_row_image = graph::build_single_graph_row_image(
                &graph,
                &image_params,
                &mut drawing_pixels,
                option.style,
                graph::GraphImageWidthMode::Fixed,
                &commit.commit_hash,
            );
            image::load_from_memory(&graph_row_image.bytes).unwrap()
        })
        .collect();

    // Create concatenated image
    let graph_width = row_images
        .iter()
        .map(|image| image.width())
        .max()
        .unwrap_or(0);
    let image_width = graph_width + 350;
    let image_height = row_images.iter().map(|image| image.height()).sum();
    let mut img_buf = image::RgbaImage::new(image_width, image_height);

    let mut y = 0;
    for (commit, image) in graph.commits.iter().zip(&row_images) {
        let height = image.height();
        // write graph
        img_buf.copy_from(image, 0, y).unwrap();

        // write hash and date
        let text = format!(
            "{} / {}",
            commit.commit_hash.as_short_hash(),
            commit.committer_date.naive_utc().format("%Y-%m-%d")
        );
        draw_text(&mut img_buf, &text, graph_width, y + (height / 4));

        // write subject
        draw_text(
            &mut img_buf,
            &commit.subject,
            graph_width,
            y + ((height / 4) * 2),
        );
        y += height;
    }

    img_buf
}

fn draw_text(img_buf: &mut image::RgbaImage, text: &str, x: u32, y: u32) {
    for (i, ch) in text.chars().enumerate() {
        let glyph = BASIC_FONTS
            .get(ch)
            .unwrap_or_else(|| panic!("unsupported graph test character: {ch:?}"));
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if bits & (1 << col) != 0 {
                    img_buf.put_pixel(
                        x + i as u32 * 8 + col,
                        y + row as u32,
                        image::Rgba([0x88, 0x88, 0x88, 0xff]),
                    );
                }
            }
        }
    }
}

fn create_output_dirs(path: &str) {
    let path = Path::new(path);
    std::fs::create_dir_all(path).unwrap();
}

pub fn copy_git_dir(path: &Path, name: &str) {
    if std::env::var_os(DUMP_GRAPH_TEST_REPOS_ENV).is_none() {
        return;
    }

    let dst_path = Path::new(OUTPUT_DIR).join(name);
    if dst_path.is_dir() {
        std::fs::remove_dir_all(&dst_path).unwrap();
    }
    copy_dir(path, &dst_path).unwrap();
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

fn compare_graph_image(
    option: &GenerateGraphOption,
    actual_img: &image::RgbaImage,
) -> Result<(), String> {
    let expected_file = format!("{}/{}.png", SNAPSHOT_DIR, option.output_name);
    let expected_img = image::open(&expected_file).unwrap();

    if actual_img.dimensions() != expected_img.dimensions() {
        return Err(format!(
            "Image dimensions are different. snapshot: {expected_file}, expected: {:?}, actual: {:?}",
            expected_img.dimensions(),
            actual_img.dimensions()
        ));
    }

    let (image_width, image_height) = actual_img.dimensions();
    let mut img_buf = image::ImageBuffer::new(image_width, image_height);
    let mut diff = false;

    for y in 0..image_height {
        for x in 0..image_width {
            let actual_pixel = *actual_img.get_pixel(x, y);
            let expected_pixel = expected_img.get_pixel(x, y);

            if actual_pixel != expected_pixel {
                img_buf.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
                diff = true;
            } else {
                img_buf.put_pixel(x, y, actual_pixel);
            }
        }
    }

    if diff {
        let diff_file = format!("{}/{}_diff.png", OUTPUT_DIR, option.output_name);
        image::save_buffer(
            diff_file.clone(),
            &img_buf,
            image_width,
            image_height,
            image::ColorType::Rgba8,
        )
        .unwrap();

        return Err(format!("Images are different. diff: {diff_file}"));
    }

    Ok(())
}
