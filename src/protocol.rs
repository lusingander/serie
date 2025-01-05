use std::env;

use base64::Engine;

// By default assume the Iterm2 is the best protocol to use for all terminals *unless* an env
// variable is set that suggests the terminal is probably Kitty.
pub fn auto_detect() -> ImageProtocol {
    // https://sw.kovidgoyal.net/kitty/glossary/#envvar-KITTY_WINDOW_ID
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return ImageProtocol::Kitty;
    }
    // https://ghostty.org/docs/help/terminfo
    if env::var("TERM").is_ok_and(|t| t == "xterm-ghostty") {
        return ImageProtocol::Kitty;
    }
    ImageProtocol::Iterm2
}

#[derive(Debug, Clone, Copy)]
pub enum ImageProtocol {
    Iterm2,
    Kitty,
}

impl ImageProtocol {
    pub fn encode(&self, bytes: &[u8], cell_width: usize) -> String {
        match self {
            ImageProtocol::Iterm2 => iterm2_encode(bytes, cell_width, 1),
            ImageProtocol::Kitty => kitty_encode(bytes, cell_width, 1),
        }
    }

    pub fn clear_line(&self, y: u16) {
        match self {
            ImageProtocol::Iterm2 => {}
            ImageProtocol::Kitty => kitty_clear_line(y),
        }
    }
}

fn to_base64_str(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

// https://iterm2.com/documentation-images.html
fn iterm2_encode(bytes: &[u8], cell_width: usize, cell_height: usize) -> String {
    format!(
        "\x1b]1337;File=size={};width={};height={};preserveAspectRatio=0;inline=1:{}\u{0007}",
        bytes.len(),
        cell_width,
        cell_height,
        to_base64_str(bytes)
    )
}

// https://sw.kovidgoyal.net/kitty/graphics-protocol/
fn kitty_encode(bytes: &[u8], cell_width: usize, cell_height: usize) -> String {
    let base64_str = to_base64_str(bytes);
    let chunk_size = 4096;

    let mut s = String::new();

    let chunks = base64_str.as_bytes().chunks(chunk_size);
    let total_chunks = chunks.len();

    for (i, chunk) in chunks.enumerate() {
        s.push_str("\x1b_Ga=d,d=C;\x1b\\");
        s.push_str("\x1b_G");
        if i == 0 {
            s.push_str(&format!("a=T,f=100,c={},r={},", cell_width, cell_height));
        }
        if i < total_chunks - 1 {
            s.push_str("m=1;");
        } else {
            s.push_str("m=0;");
        }
        s.push_str(std::str::from_utf8(chunk).unwrap());
        s.push_str("\x1b\\");
    }

    s
}

fn kitty_clear_line(y: u16) {
    let y = y + 1; // 1-based
    print!("\x1b_Ga=d,d=P,x=1,y={y};\x1b\\");
}
