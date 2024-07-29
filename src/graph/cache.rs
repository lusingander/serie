use std::path::PathBuf;

use olpc_cjson::CanonicalFormatter;
use serde::Serialize;
use sha1::{Digest, Sha1};

use crate::graph::{Edge, GraphRowImage};

const APP_DIR_NAME: &str = "serie";

#[derive(Debug, Serialize)]
pub struct ImageCacheDirKey {
    width: u16,
    height: u16,
    line_width: u16,
    circle_inner_radius: u16,
    circle_outer_radius: u16,
    edge_colors: Vec<[u8; 4]>,
}

impl ImageCacheDirKey {
    pub fn new(
        width: u16,
        height: u16,
        line_width: u16,
        circle_inner_radius: u16,
        circle_outer_radius: u16,
        edge_colors: Vec<image::Rgba<u8>>,
    ) -> Self {
        Self {
            width,
            height,
            line_width,
            circle_inner_radius,
            circle_outer_radius,
            edge_colors: edge_colors.iter().map(|c| c.0).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ImageCacheFileKey {
    pos_x: usize,
    cell_count: usize,
    edges: Vec<[usize; 3]>,
}

impl ImageCacheFileKey {
    pub fn new(pos_x: usize, cell_count: usize, edges: Vec<Edge>) -> Self {
        Self {
            pos_x,
            cell_count,
            edges: edges
                .iter()
                .map(|e| [e.edge_type as u8 as usize, e.pos_x, e.associated_line_pos_x])
                .collect(),
        }
    }
}

pub struct ImageCache {
    cache_dir: PathBuf,
}

impl ImageCache {
    pub fn new(key: ImageCacheDirKey) -> Self {
        let cache_dir = cache_dir(&key);
        Self { cache_dir }
    }

    pub fn load_image_cache(&self, key: &ImageCacheFileKey) -> Option<GraphRowImage> {
        let cache_file_path = self.image_cache_file_path(key);
        if cache_file_path.exists() {
            let bytes = std::fs::read(cache_file_path).unwrap();
            let image = GraphRowImage {
                bytes,
                cell_count: key.cell_count,
            };
            Some(image)
        } else {
            None
        }
    }

    pub fn save_image_cache(&self, key: &ImageCacheFileKey, image: &GraphRowImage) {
        let cache_file_path = self.image_cache_file_path(key);
        std::fs::write(cache_file_path, &image.bytes).unwrap();
    }

    fn image_cache_file_path(&self, key: &ImageCacheFileKey) -> PathBuf {
        let cache_file_name = format!("{}.png", hash_str(key));
        self.cache_dir.join(cache_file_name)
    }
}

fn cache_dir(key: &ImageCacheDirKey) -> PathBuf {
    // $XDG_CACHE_HOME/{APP_DIR_NAME}/{hash of key}
    xdg::BaseDirectories::with_prefix(APP_DIR_NAME)
        .unwrap()
        .create_cache_directory(hash_str(key))
        .unwrap()
}

fn hash_str<T: Serialize>(t: T) -> String {
    let mut buf = Vec::new();
    let mut serializer =
        serde_json::Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
    t.serialize(&mut serializer).unwrap();
    format!("{:x}", Sha1::digest(buf))
}
