use base64::Engine;
use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};
use once_cell::sync::OnceCell;
use ratatui::crossterm::cursor::RestorePosition;
use ratatui::crossterm::cursor::SavePosition;
use ratatui::crossterm::execute;
use ratatui::crossterm::style::Print;
use ratatui::crossterm::terminal::disable_raw_mode;
use ratatui::crossterm::terminal::enable_raw_mode;
use std::env;
use std::io;
use std::io::stdout;
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

// Use kitty graphics protocol if we can detect it, otherwise fall back to iTerm2 protocol.
pub fn auto_detect() -> ImageProtocol {
    let passthru = PassthruProtocol::detect();
    if let Ok(true) = check_kitty_support(passthru) {
        return ImageProtocol::Kitty { passthru };
    }
    ImageProtocol::Iterm2
}

#[derive(Debug, Clone, Copy)]
pub enum PassthruProtocol {
    Tmux,
    NoPassthru,
}

impl PassthruProtocol {
    pub fn detect() -> Self {
        if env::var("TERM").is_ok_and(|term| term.starts_with("tmux"))
            || env::var("TERM_PROGRAM").is_ok_and(|term_program| term_program == "tmux")
        {
            return Self::Tmux;
        }

        Self::NoPassthru
    }

    fn escape_strings(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::NoPassthru => ("", "\x1b", ""),
            // Tmux requires escapes to be escaped, and some special start/end sequences.
            Self::Tmux => ("\x1bPtmux;", "\x1b\x1b", "\x1b\\"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImageProtocol {
    Iterm2,
    Kitty { passthru: PassthruProtocol },
}

impl ImageProtocol {
    pub fn encode(&self, bytes: &[u8], cell_width: usize) -> String {
        match self {
            ImageProtocol::Iterm2 => iterm2_encode(bytes, cell_width, 1),
            ImageProtocol::Kitty { passthru } => kitty_encode(bytes, cell_width, 1, *passthru),
        }
    }

    pub fn clear_line(&self, y: u16) {
        match self {
            ImageProtocol::Iterm2 => {}
            ImageProtocol::Kitty { passthru } => kitty_clear_line(y, *passthru),
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

#[rustfmt::skip]
static KITTY_DIACRITICS: [char; 297] = [
    '\u{0305}', '\u{030D}', '\u{030E}', '\u{0310}', '\u{0312}', '\u{033D}', '\u{033E}', '\u{033F}', '\u{0346}', '\u{034A}', '\u{034B}',
    '\u{034C}', '\u{0350}', '\u{0351}', '\u{0352}', '\u{0357}', '\u{035B}', '\u{0363}', '\u{0364}', '\u{0365}', '\u{0366}', '\u{0367}',
    '\u{0368}', '\u{0369}', '\u{036A}', '\u{036B}', '\u{036C}', '\u{036D}', '\u{036E}', '\u{036F}', '\u{0483}', '\u{0484}', '\u{0485}',
    '\u{0486}', '\u{0487}', '\u{0592}', '\u{0593}', '\u{0594}', '\u{0595}', '\u{0597}', '\u{0598}', '\u{0599}', '\u{059C}', '\u{059D}',
    '\u{059E}', '\u{059F}', '\u{05A0}', '\u{05A1}', '\u{05A8}', '\u{05A9}', '\u{05AB}', '\u{05AC}', '\u{05AF}', '\u{05C4}', '\u{0610}',
    '\u{0611}', '\u{0612}', '\u{0613}', '\u{0614}', '\u{0615}', '\u{0616}', '\u{0617}', '\u{0657}', '\u{0658}', '\u{0659}', '\u{065A}',
    '\u{065B}', '\u{065D}', '\u{065E}', '\u{06D6}', '\u{06D7}', '\u{06D8}', '\u{06D9}', '\u{06DA}', '\u{06DB}', '\u{06DC}', '\u{06DF}',
    '\u{06E0}', '\u{06E1}', '\u{06E2}', '\u{06E4}', '\u{06E7}', '\u{06E8}', '\u{06EB}', '\u{06EC}', '\u{0730}', '\u{0732}', '\u{0733}',
    '\u{0735}', '\u{0736}', '\u{073A}', '\u{073D}', '\u{073F}', '\u{0740}', '\u{0741}', '\u{0743}', '\u{0745}', '\u{0747}', '\u{0749}',
    '\u{074A}', '\u{07EB}', '\u{07EC}', '\u{07ED}', '\u{07EE}', '\u{07EF}', '\u{07F0}', '\u{07F1}', '\u{07F3}', '\u{0816}', '\u{0817}',
    '\u{0818}', '\u{0819}', '\u{081B}', '\u{081C}', '\u{081D}', '\u{081E}', '\u{081F}', '\u{0820}', '\u{0821}', '\u{0822}', '\u{0823}',
    '\u{0825}', '\u{0826}', '\u{0827}', '\u{0829}', '\u{082A}', '\u{082B}', '\u{082C}', '\u{082D}', '\u{0951}', '\u{0953}', '\u{0954}',
    '\u{0F82}', '\u{0F83}', '\u{0F86}', '\u{0F87}', '\u{135D}', '\u{135E}', '\u{135F}', '\u{17DD}', '\u{193A}', '\u{1A17}', '\u{1A75}',
    '\u{1A76}', '\u{1A77}', '\u{1A78}', '\u{1A79}', '\u{1A7A}', '\u{1A7B}', '\u{1A7C}', '\u{1B6B}', '\u{1B6D}', '\u{1B6E}', '\u{1B6F}',
    '\u{1B70}', '\u{1B71}', '\u{1B72}', '\u{1B73}', '\u{1CD0}', '\u{1CD1}', '\u{1CD2}', '\u{1CDA}', '\u{1CDB}', '\u{1CE0}', '\u{1DC0}',
    '\u{1DC1}', '\u{1DC3}', '\u{1DC4}', '\u{1DC5}', '\u{1DC6}', '\u{1DC7}', '\u{1DC8}', '\u{1DC9}', '\u{1DCB}', '\u{1DCC}', '\u{1DD1}',
    '\u{1DD2}', '\u{1DD3}', '\u{1DD4}', '\u{1DD5}', '\u{1DD6}', '\u{1DD7}', '\u{1DD8}', '\u{1DD9}', '\u{1DDA}', '\u{1DDB}', '\u{1DDC}',
    '\u{1DDD}', '\u{1DDE}', '\u{1DDF}', '\u{1DE0}', '\u{1DE1}', '\u{1DE2}', '\u{1DE3}', '\u{1DE4}', '\u{1DE5}', '\u{1DE6}', '\u{1DFE}',
    '\u{20D0}', '\u{20D1}', '\u{20D4}', '\u{20D5}', '\u{20D6}', '\u{20D7}', '\u{20DB}', '\u{20DC}', '\u{20E1}', '\u{20E7}', '\u{20E9}',
    '\u{20F0}', '\u{2CEF}', '\u{2CF0}', '\u{2CF1}', '\u{2DE0}', '\u{2DE1}', '\u{2DE2}', '\u{2DE3}', '\u{2DE4}', '\u{2DE5}', '\u{2DE6}',
    '\u{2DE7}', '\u{2DE8}', '\u{2DE9}', '\u{2DEA}', '\u{2DEB}', '\u{2DEC}', '\u{2DED}', '\u{2DEE}', '\u{2DEF}', '\u{2DF0}', '\u{2DF1}',
    '\u{2DF2}', '\u{2DF3}', '\u{2DF4}', '\u{2DF5}', '\u{2DF6}', '\u{2DF7}', '\u{2DF8}', '\u{2DF9}', '\u{2DFA}', '\u{2DFB}', '\u{2DFC}',
    '\u{2DFD}', '\u{2DFE}', '\u{2DFF}', '\u{A66F}', '\u{A67C}', '\u{A67D}', '\u{A6F0}', '\u{A6F1}', '\u{A8E0}', '\u{A8E1}', '\u{A8E2}',
    '\u{A8E3}', '\u{A8E4}', '\u{A8E5}', '\u{A8E6}', '\u{A8E7}', '\u{A8E8}', '\u{A8E9}', '\u{A8EA}', '\u{A8EB}', '\u{A8EC}', '\u{A8ED}',
    '\u{A8EE}', '\u{A8EF}', '\u{A8F0}', '\u{A8F1}', '\u{AAB0}', '\u{AAB2}', '\u{AAB3}', '\u{AAB7}', '\u{AAB8}', '\u{AABE}', '\u{AABF}',
    '\u{AAC1}', '\u{FE20}', '\u{FE21}', '\u{FE22}', '\u{FE23}', '\u{FE24}', '\u{FE25}', '\u{FE26}', '\u{10A0F}', '\u{10A38}', '\u{1D185}',
    '\u{1D186}', '\u{1D187}', '\u{1D188}', '\u{1D189}', '\u{1D1AA}', '\u{1D1AB}', '\u{1D1AC}', '\u{1D1AD}', '\u{1D242}', '\u{1D243}', '\u{1D244}',
];

fn kitty_encode(
    bytes: &[u8],
    cell_width: usize,
    cell_height: usize,
    passthru: PassthruProtocol,
) -> String {
    let base64_str = to_base64_str(bytes);
    let chunk_size = 4096;

    let (start, escape, end) = passthru.escape_strings();

    let mut s = String::new();

    s.push_str(start);

    let chunks = base64_str.as_bytes().chunks(chunk_size);
    let total_chunks = chunks.len();

    let id = kitty_image_id();
    for (i, chunk) in chunks.enumerate() {
        s.push_str(&format!("{escape}_G"));
        if i == 0 {
            s.push_str(&format!(
                "q=2,a=T,f=100,C=1,U=1,c={cell_width},r={cell_height},i={id},"
            ));
        }
        if i < total_chunks - 1 {
            s.push_str("m=1;");
        } else {
            s.push_str("m=0;");
        }
        s.push_str(std::str::from_utf8(chunk).unwrap());
        s.push_str(&format!("{escape}\\"));
    }
    s.push_str(end);

    let (id_diacritic, id_r, id_g, id_b) = (
        (id >> 24) & 0xff,
        (id >> 16) & 0xff,
        (id >> 8) & 0xff,
        id & 0xff,
    );
    s.push_str(&format!("\x1b[38;2;{id_r};{id_g};{id_b}m"));

    for y in 0..cell_height {
        for x in 0..cell_width {
            s.push('\u{10EEEE}');

            s.push_str(&format!(
                "{}",
                *KITTY_DIACRITICS.get(y).unwrap_or(&KITTY_DIACRITICS[0])
            ));

            s.push_str(&format!(
                "{}",
                *KITTY_DIACRITICS.get(x).unwrap_or(&KITTY_DIACRITICS[0])
            ));

            s.push_str(&format!(
                "{}",
                *KITTY_DIACRITICS
                    .get(id_diacritic as usize)
                    .unwrap_or(&KITTY_DIACRITICS[0])
            ));
        }
    }
    s
}

fn kitty_clear_line(y: u16, passthru: PassthruProtocol) {
    let y = y + 1; // 1-based
    let (start, escape, end) = passthru.escape_strings();
    print!("{start}{escape}_Ga=d,d=Y,y={y};{escape}\\{end}");
}

// If the app is rerun with diffrent images sometimes ghostty resuses the image with the same id
// from the previous run, so tie in the PID with the id
fn kitty_image_id() -> u32 {
    static COUNTER: AtomicU16 = AtomicU16::new(1);
    let counter = COUNTER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
            Some(x.wrapping_add(1))
        })
        .unwrap();

    static PID_CACHE: OnceCell<u16> = OnceCell::new();
    let pid = PID_CACHE.get_or_init(|| std::process::id() as u16);

    ((*pid as u32) << 16) | (counter as u32)
}

struct RawStdIn {
    stdin_fd: i32,
    original_flags: i32,
}

impl RawStdIn {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let stdin_fd = std::io::stdin().as_raw_fd();
        let original_flags = unsafe { fcntl(stdin_fd, F_GETFL) };
        unsafe { fcntl(stdin_fd, F_SETFL, original_flags | O_NONBLOCK) };
        Ok(RawStdIn {
            stdin_fd,
            original_flags,
        })
    }
}

impl Drop for RawStdIn {
    fn drop(&mut self) {
        unsafe { fcntl(self.stdin_fd, F_SETFL, self.original_flags) };
        disable_raw_mode().ok();
    }
}

// https://sw.kovidgoyal.net/kitty/graphics-protocol/#querying-support-and-available-transmission-mediums
// All terminals emulators should respond to device attribute query, as a backup
// read in raw mode so we can timeout if no response is received
pub fn check_kitty_support(passthru: PassthruProtocol) -> io::Result<bool> {
    let (start, escape, end) = passthru.escape_strings();
    let device_attr_query = format!("{start}{escape}[0c{end}");
    let kitty_support_query =
        format!("{start}{escape}_Gi=9999,s=1,v=1,a=q,t=d,f=24;AAAA{escape}\\{end}");

    let _raw_stdin_guard = RawStdIn::new()?;

    execute!(
        stdout(),
        SavePosition,
        Print(kitty_support_query),
        Print(device_attr_query),
        RestorePosition
    )?;

    let stdin = io::stdin();
    let mut response = Vec::new();
    let mut buffer = [0u8; 1];
    let start = Instant::now();

    loop {
        if start.elapsed() > Duration::from_millis(500) {
            break;
        }

        match stdin.lock().read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let byte = buffer[0];
                response.push(byte);
                if byte == b'c'
                    && response.contains(&0x1b)
                    && response
                        .rsplitn(2, |&b| b == 0x1b)
                        .next()
                        .is_some_and(|s| s.starts_with(b"[?"))
                {
                    break;
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    let response = String::from_utf8_lossy(&response).to_string();
    Ok(response.contains("\x1b_Gi=9999;OK"))
}
