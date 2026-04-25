use std::env;
use std::io::{self, Write};

use base64::Engine;
use ratatui::style::Color;
use ratatui::style::Style;

// By default assume the Iterm2 is the best protocol to use for all terminals *unless* an env
// variable is set that suggests the terminal is probably Kitty.
pub fn auto_detect() -> ImageProtocol {
    // https://sw.kovidgoyal.net/kitty/glossary/#envvar-KITTY_WINDOW_ID
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return ImageProtocol::Kitty;
    }
    // https://ghostty.org/docs/help/terminfo
    if env::var("TERM").is_ok_and(|t| t == "xterm-ghostty")
        || env::var("GHOSTTY_RESOURCES_DIR").is_ok()
    {
        return ImageProtocol::Kitty;
    }
    ImageProtocol::Iterm2
}

#[derive(Debug, Clone, Copy)]
pub enum ImageProtocol {
    Iterm2,
    Kitty,
    KittyUnicode,
}

#[derive(Debug, Clone)]
pub struct PreparedImageCell {
    symbol: String,
    style: Style,
    skip: bool,
}

impl PreparedImageCell {
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn skip(&self) -> bool {
        self.skip
    }
}

#[derive(Debug, Clone)]
pub struct PreparedImage {
    cells: Vec<PreparedImageCell>,
    cell_width: usize,
    upload_data: Option<String>,
}

impl PreparedImage {
    pub fn cells(&self) -> &[PreparedImageCell] {
        &self.cells
    }

    pub fn cell_width(&self) -> usize {
        self.cell_width
    }

    pub fn take_upload_data(&mut self) -> Option<String> {
        self.upload_data.take()
    }
}

impl ImageProtocol {
    pub fn prepare_image(&self, bytes: &[u8], cell_width: usize, image_id: u32) -> PreparedImage {
        let symbol = match self {
            ImageProtocol::Iterm2 => iterm2_encode(bytes, cell_width, 1),
            ImageProtocol::Kitty => kitty_encode(bytes, cell_width, 1),
            ImageProtocol::KittyUnicode => {
                return kitty_unicode_prepare(bytes, cell_width, image_id);
            }
        };
        let mut cells = Vec::with_capacity(cell_width);
        cells.push(PreparedImageCell {
            symbol,
            style: Style::default(),
            skip: false,
        });
        for _ in 1..cell_width {
            cells.push(PreparedImageCell {
                symbol: String::new(),
                style: Style::default(),
                skip: true,
            });
        }
        PreparedImage {
            cells,
            cell_width,
            upload_data: None,
        }
    }

    pub fn clear_line(&self, y: u16) {
        match self {
            ImageProtocol::Iterm2 => {}
            ImageProtocol::Kitty => kitty_clear_line(y),
            ImageProtocol::KittyUnicode => {}
        }
    }

    pub fn clear(&self) {
        match self {
            ImageProtocol::Iterm2 => {}
            ImageProtocol::Kitty => kitty_clear(),
            ImageProtocol::KittyUnicode => {}
        }
    }

    pub fn delete_images(&self, image_ids: &[u32]) -> Result<(), std::io::Error> {
        match self {
            ImageProtocol::Iterm2 | ImageProtocol::Kitty => Ok(()),
            ImageProtocol::KittyUnicode => kitty_unicode_delete_images(image_ids),
        }
    }
}

const KITTY_PLACEHOLDER: char = '\u{10EEEE}';
static ROW_COLUMN_DIACRITICS: &[char] = &[
    '\u{0305}',
    '\u{030D}',
    '\u{030E}',
    '\u{0310}',
    '\u{0312}',
    '\u{033D}',
    '\u{033E}',
    '\u{033F}',
    '\u{0346}',
    '\u{034A}',
    '\u{034B}',
    '\u{034C}',
    '\u{0350}',
    '\u{0351}',
    '\u{0352}',
    '\u{0357}',
    '\u{035B}',
    '\u{0363}',
    '\u{0364}',
    '\u{0365}',
    '\u{0366}',
    '\u{0367}',
    '\u{0368}',
    '\u{0369}',
    '\u{036A}',
    '\u{036B}',
    '\u{036C}',
    '\u{036D}',
    '\u{036E}',
    '\u{036F}',
    '\u{0483}',
    '\u{0484}',
    '\u{0485}',
    '\u{0486}',
    '\u{0487}',
    '\u{0592}',
    '\u{0593}',
    '\u{0594}',
    '\u{0595}',
    '\u{0597}',
    '\u{0598}',
    '\u{0599}',
    '\u{059C}',
    '\u{059D}',
    '\u{059E}',
    '\u{059F}',
    '\u{05A0}',
    '\u{05A1}',
    '\u{05A8}',
    '\u{05A9}',
    '\u{05AB}',
    '\u{05AC}',
    '\u{05AF}',
    '\u{05C4}',
    '\u{0610}',
    '\u{0611}',
    '\u{0612}',
    '\u{0613}',
    '\u{0614}',
    '\u{0615}',
    '\u{0616}',
    '\u{0617}',
    '\u{0657}',
    '\u{0658}',
    '\u{0659}',
    '\u{065A}',
    '\u{065B}',
    '\u{065D}',
    '\u{065E}',
    '\u{06D6}',
    '\u{06D7}',
    '\u{06D8}',
    '\u{06D9}',
    '\u{06DA}',
    '\u{06DB}',
    '\u{06DC}',
    '\u{06DF}',
    '\u{06E0}',
    '\u{06E1}',
    '\u{06E2}',
    '\u{06E4}',
    '\u{06E7}',
    '\u{06E8}',
    '\u{06EB}',
    '\u{06EC}',
    '\u{0730}',
    '\u{0732}',
    '\u{0733}',
    '\u{0735}',
    '\u{0736}',
    '\u{073A}',
    '\u{073D}',
    '\u{073F}',
    '\u{0740}',
    '\u{0741}',
    '\u{0743}',
    '\u{0745}',
    '\u{0747}',
    '\u{0749}',
    '\u{074A}',
    '\u{07EB}',
    '\u{07EC}',
    '\u{07ED}',
    '\u{07EE}',
    '\u{07EF}',
    '\u{07F0}',
    '\u{07F1}',
    '\u{07F3}',
    '\u{0816}',
    '\u{0817}',
    '\u{0818}',
    '\u{0819}',
    '\u{081B}',
    '\u{081C}',
    '\u{081D}',
    '\u{081E}',
    '\u{081F}',
    '\u{0820}',
    '\u{0821}',
    '\u{0822}',
    '\u{0823}',
    '\u{0825}',
    '\u{0826}',
    '\u{0827}',
    '\u{0829}',
    '\u{082A}',
    '\u{082B}',
    '\u{082C}',
    '\u{082D}',
    '\u{0951}',
    '\u{0953}',
    '\u{0954}',
    '\u{0F82}',
    '\u{0F83}',
    '\u{0F86}',
    '\u{0F87}',
    '\u{135D}',
    '\u{135E}',
    '\u{135F}',
    '\u{17DD}',
    '\u{193A}',
    '\u{1A17}',
    '\u{1A75}',
    '\u{1A76}',
    '\u{1A77}',
    '\u{1A78}',
    '\u{1A79}',
    '\u{1A7A}',
    '\u{1A7B}',
    '\u{1A7C}',
    '\u{1B6B}',
    '\u{1B6D}',
    '\u{1B6E}',
    '\u{1B6F}',
    '\u{1B70}',
    '\u{1B71}',
    '\u{1B72}',
    '\u{1B73}',
    '\u{1CD0}',
    '\u{1CD1}',
    '\u{1CD2}',
    '\u{1CDA}',
    '\u{1CDB}',
    '\u{1CE0}',
    '\u{1DC0}',
    '\u{1DC1}',
    '\u{1DC3}',
    '\u{1DC4}',
    '\u{1DC5}',
    '\u{1DC6}',
    '\u{1DC7}',
    '\u{1DC8}',
    '\u{1DC9}',
    '\u{1DCB}',
    '\u{1DCC}',
    '\u{1DD1}',
    '\u{1DD2}',
    '\u{1DD3}',
    '\u{1DD4}',
    '\u{1DD5}',
    '\u{1DD6}',
    '\u{1DD7}',
    '\u{1DD8}',
    '\u{1DD9}',
    '\u{1DDA}',
    '\u{1DDB}',
    '\u{1DDC}',
    '\u{1DDD}',
    '\u{1DDE}',
    '\u{1DDF}',
    '\u{1DE0}',
    '\u{1DE1}',
    '\u{1DE2}',
    '\u{1DE3}',
    '\u{1DE4}',
    '\u{1DE5}',
    '\u{1DE6}',
    '\u{1DFE}',
    '\u{20D0}',
    '\u{20D1}',
    '\u{20D4}',
    '\u{20D5}',
    '\u{20D6}',
    '\u{20D7}',
    '\u{20DB}',
    '\u{20DC}',
    '\u{20E1}',
    '\u{20E7}',
    '\u{20E9}',
    '\u{20F0}',
    '\u{2CEF}',
    '\u{2CF0}',
    '\u{2CF1}',
    '\u{2DE0}',
    '\u{2DE1}',
    '\u{2DE2}',
    '\u{2DE3}',
    '\u{2DE4}',
    '\u{2DE5}',
    '\u{2DE6}',
    '\u{2DE7}',
    '\u{2DE8}',
    '\u{2DE9}',
    '\u{2DEA}',
    '\u{2DEB}',
    '\u{2DEC}',
    '\u{2DED}',
    '\u{2DEE}',
    '\u{2DEF}',
    '\u{2DF0}',
    '\u{2DF1}',
    '\u{2DF2}',
    '\u{2DF3}',
    '\u{2DF4}',
    '\u{2DF5}',
    '\u{2DF6}',
    '\u{2DF7}',
    '\u{2DF8}',
    '\u{2DF9}',
    '\u{2DFA}',
    '\u{2DFB}',
    '\u{2DFC}',
    '\u{2DFD}',
    '\u{2DFE}',
    '\u{2DFF}',
    '\u{A66F}',
    '\u{A67C}',
    '\u{A67D}',
    '\u{A6F0}',
    '\u{A6F1}',
    '\u{A8E0}',
    '\u{A8E1}',
    '\u{A8E2}',
    '\u{A8E3}',
    '\u{A8E4}',
    '\u{A8E5}',
    '\u{A8E6}',
    '\u{A8E7}',
    '\u{A8E8}',
    '\u{A8E9}',
    '\u{A8EA}',
    '\u{A8EB}',
    '\u{A8EC}',
    '\u{A8ED}',
    '\u{A8EE}',
    '\u{A8EF}',
    '\u{A8F0}',
    '\u{A8F1}',
    '\u{AAB0}',
    '\u{AAB2}',
    '\u{AAB3}',
    '\u{AAB7}',
    '\u{AAB8}',
    '\u{AABE}',
    '\u{AABF}',
    '\u{AAC1}',
    '\u{FE20}',
    '\u{FE21}',
    '\u{FE22}',
    '\u{FE23}',
    '\u{FE24}',
    '\u{FE25}',
    '\u{FE26}',
    '\u{10A0F}',
    '\u{10A38}',
    '\u{1D185}',
    '\u{1D186}',
    '\u{1D187}',
    '\u{1D188}',
    '\u{1D189}',
    '\u{1D1AA}',
    '\u{1D1AB}',
    '\u{1D1AC}',
    '\u{1D1AD}',
    '\u{1D242}',
    '\u{1D243}',
    '\u{1D244}',
];

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

    s.push_str("\x1b_Ga=d,d=C;\x1b\\");
    for (i, chunk) in chunks.enumerate() {
        s.push_str("\x1b_G");
        if i == 0 {
            s.push_str(&format!("a=T,f=100,c={cell_width},r={cell_height},"));
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

fn kitty_unicode_prepare(bytes: &[u8], cell_width: usize, image_id: u32) -> PreparedImage {
    let mut cells = Vec::with_capacity(cell_width);
    let upload_symbol = kitty_unicode_encode(bytes, cell_width, 1, image_id);
    let foreground = Color::Rgb(
        ((image_id >> 16) & 0xff) as u8,
        ((image_id >> 8) & 0xff) as u8,
        (image_id & 0xff) as u8,
    );
    let image_id_msb = ((image_id >> 24) & 0xff) as usize;

    for column in 0..cell_width {
        let symbol = [
            KITTY_PLACEHOLDER,
            row_column_diacritic(0),
            row_column_diacritic(column),
            row_column_diacritic(image_id_msb),
        ]
        .into_iter()
        .collect();

        cells.push(PreparedImageCell {
            symbol,
            style: Style::default().fg(foreground),
            skip: false,
        });
    }

    PreparedImage {
        cells,
        cell_width,
        upload_data: Some(upload_symbol),
    }
}

fn kitty_unicode_encode(
    bytes: &[u8],
    cell_width: usize,
    cell_height: usize,
    image_id: u32,
) -> String {
    let base64_str = to_base64_str(bytes);
    let chunk_size = 4096;

    let mut s = String::new();

    let chunks = base64_str.as_bytes().chunks(chunk_size);
    let total_chunks = chunks.len();

    for (i, chunk) in chunks.enumerate() {
        s.push_str("\x1b_G");
        if i == 0 {
            s.push_str(&format!(
                "a=T,f=100,U=1,q=2,i={image_id},c={cell_width},r={cell_height},"
            ));
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

fn row_column_diacritic(index: usize) -> char {
    ROW_COLUMN_DIACRITICS[index]
}

fn kitty_clear_line(y: u16) {
    let y = y + 1; // 1-based
    print!("\x1b_Ga=d,d=Y,y={y};\x1b\\");
}

fn kitty_clear() {
    print!("\x1b_Ga=d,d=A;\x1b\\");
}

fn kitty_unicode_delete_images(image_ids: &[u32]) -> Result<(), io::Error> {
    if image_ids.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout().lock();
    for image_id in image_ids {
        write!(stdout, "\x1b_Ga=d,d=I,i={image_id}\x1b\\")?;
    }
    stdout.flush()
}
