use clap::ValueEnum;
use ratatui::style::Color as RatatuiColor;
use serde::Deserialize;
use smart_default::SmartDefault;
use umbra::optional;

use crate::config::GraphColorConfig;

/// Whether the page-padding band is lighter or darker than the content `bg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum PaddingShade {
    #[default]
    Lighter,
    Darker,
}

#[optional(derives = [Deserialize], visibility = pub)]
#[derive(Debug, Clone, PartialEq, Eq, SmartDefault)]
pub struct ColorTheme {
    #[default(RatatuiColor::Reset)]
    pub fg: RatatuiColor,
    #[default(RatatuiColor::Reset)]
    pub bg: RatatuiColor,

    #[default(RatatuiColor::White)]
    pub list_selected_fg: RatatuiColor,
    #[default(RatatuiColor::DarkGray)]
    pub list_selected_bg: RatatuiColor,
    #[default(RatatuiColor::Yellow)]
    pub list_ref_paren_fg: RatatuiColor,
    #[default(RatatuiColor::Green)]
    pub list_ref_branch_fg: RatatuiColor,
    #[default(RatatuiColor::Red)]
    pub list_ref_remote_branch_fg: RatatuiColor,
    #[default(RatatuiColor::Yellow)]
    pub list_ref_tag_fg: RatatuiColor,
    #[default(RatatuiColor::Magenta)]
    pub list_ref_stash_fg: RatatuiColor,
    #[default(RatatuiColor::Cyan)]
    pub list_head_fg: RatatuiColor,
    #[default(RatatuiColor::Reset)]
    pub list_subject_fg: RatatuiColor,
    #[default(RatatuiColor::Cyan)]
    pub list_name_fg: RatatuiColor,
    #[default(RatatuiColor::Yellow)]
    pub list_hash_fg: RatatuiColor,
    #[default(RatatuiColor::Magenta)]
    pub list_date_fg: RatatuiColor,
    #[default(RatatuiColor::Black)]
    pub list_match_fg: RatatuiColor,
    #[default(RatatuiColor::Yellow)]
    pub list_match_bg: RatatuiColor,

    #[default(RatatuiColor::Reset)]
    pub detail_label_fg: RatatuiColor,
    #[default(RatatuiColor::Reset)]
    pub detail_name_fg: RatatuiColor,
    #[default(RatatuiColor::Reset)]
    pub detail_date_fg: RatatuiColor,
    #[default(RatatuiColor::Blue)]
    pub detail_email_fg: RatatuiColor,
    #[default(RatatuiColor::Reset)]
    pub detail_hash_fg: RatatuiColor,
    #[default(RatatuiColor::Green)]
    pub detail_ref_branch_fg: RatatuiColor,
    #[default(RatatuiColor::Red)]
    pub detail_ref_remote_branch_fg: RatatuiColor,
    #[default(RatatuiColor::Yellow)]
    pub detail_ref_tag_fg: RatatuiColor,
    #[default(RatatuiColor::Green)]
    pub detail_file_change_add_fg: RatatuiColor,
    #[default(RatatuiColor::Yellow)]
    pub detail_file_change_modify_fg: RatatuiColor,
    #[default(RatatuiColor::Red)]
    pub detail_file_change_delete_fg: RatatuiColor,
    #[default(RatatuiColor::Magenta)]
    pub detail_file_change_move_fg: RatatuiColor,

    #[default(RatatuiColor::White)]
    pub ref_selected_fg: RatatuiColor,
    #[default(RatatuiColor::DarkGray)]
    pub ref_selected_bg: RatatuiColor,

    #[default(RatatuiColor::Green)]
    pub help_block_title_fg: RatatuiColor,
    #[default(RatatuiColor::Yellow)]
    pub help_key_fg: RatatuiColor,

    #[default(RatatuiColor::Reset)]
    pub virtual_cursor_fg: RatatuiColor,
    #[default(RatatuiColor::Reset)]
    pub status_input_fg: RatatuiColor,
    #[default(RatatuiColor::DarkGray)]
    pub status_input_transient_fg: RatatuiColor,
    #[default(RatatuiColor::Cyan)]
    pub status_info_fg: RatatuiColor,
    #[default(RatatuiColor::Green)]
    pub status_success_fg: RatatuiColor,
    #[default(RatatuiColor::Yellow)]
    pub status_warn_fg: RatatuiColor,
    #[default(RatatuiColor::Red)]
    pub status_error_fg: RatatuiColor,

    #[default(RatatuiColor::DarkGray)]
    pub divider_fg: RatatuiColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl GraphColor {
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }

    pub fn to_image_color(self) -> image::Rgba<u8> {
        image::Rgba([self.r, self.g, self.b, self.a])
    }

    pub fn to_ratatui_color(self) -> RatatuiColor {
        RatatuiColor::Rgb(self.r, self.g, self.b)
    }

    fn transparent() -> Self {
        Self::from_rgba(0, 0, 0, 0)
    }
}

#[derive(Debug, Clone)]
pub struct GraphColorSet {
    pub colors: Vec<GraphColor>,
    pub edge_color: GraphColor,
    pub background_color: GraphColor,
}

impl GraphColorSet {
    pub fn new(config: &GraphColorConfig) -> Self {
        let colors = config
            .branches
            .iter()
            .filter_map(|s| parse_rgba_color(s))
            .collect();
        let edge_color = parse_rgba_color(&config.edge).unwrap_or(GraphColor::transparent());
        let background_color =
            parse_rgba_color(&config.background).unwrap_or(GraphColor::transparent());

        Self {
            colors,
            edge_color,
            background_color,
        }
    }

    pub fn get(&self, index: usize) -> GraphColor {
        self.colors[index % self.colors.len()]
    }
}

/// Background for the page-padding band, shifted from `bg` toward white or black.
pub(crate) fn padding_bg(bg: RatatuiColor, shade: PaddingShade) -> RatatuiColor {
    const STEP: u8 = 96;
    match rgb_of(bg) {
        None => match shade {
            PaddingShade::Lighter => RatatuiColor::Rgb(STEP, STEP, STEP),
            PaddingShade::Darker => RatatuiColor::Black,
        },
        Some((r, g, b)) => {
            let (r, g, b) = match shade {
                PaddingShade::Lighter => (lift(r, STEP), lift(g, STEP), lift(b, STEP)),
                PaddingShade::Darker => (drop(r, STEP), drop(g, STEP), drop(b, STEP)),
            };
            RatatuiColor::Rgb(r, g, b)
        }
    }
}

fn lift(c: u8, d: u8) -> u8 {
    c.saturating_add(d)
}

fn drop(c: u8, d: u8) -> u8 {
    c.saturating_sub(d)
}

fn rgb_of(color: RatatuiColor) -> Option<(u8, u8, u8)> {
    match color {
        RatatuiColor::Reset => None,
        RatatuiColor::Black => Some((0, 0, 0)),
        RatatuiColor::Red => Some((128, 0, 0)),
        RatatuiColor::Green => Some((0, 128, 0)),
        RatatuiColor::Yellow => Some((128, 128, 0)),
        RatatuiColor::Blue => Some((0, 0, 128)),
        RatatuiColor::Magenta => Some((128, 0, 128)),
        RatatuiColor::Cyan => Some((0, 128, 128)),
        RatatuiColor::Gray => Some((192, 192, 192)),
        RatatuiColor::DarkGray => Some((128, 128, 128)),
        RatatuiColor::LightRed => Some((255, 0, 0)),
        RatatuiColor::LightGreen => Some((0, 255, 0)),
        RatatuiColor::LightYellow => Some((255, 255, 0)),
        RatatuiColor::LightBlue => Some((0, 0, 255)),
        RatatuiColor::LightMagenta => Some((255, 0, 255)),
        RatatuiColor::LightCyan => Some((0, 255, 255)),
        RatatuiColor::White => Some((255, 255, 255)),
        RatatuiColor::Rgb(r, g, b) => Some((r, g, b)),
        RatatuiColor::Indexed(i) => Some(indexed_rgb(i)),
    }
}

fn indexed_rgb(i: u8) -> (u8, u8, u8) {
    match i {
        0 => (0, 0, 0),
        1 => (128, 0, 0),
        2 => (0, 128, 0),
        3 => (128, 128, 0),
        4 => (0, 0, 128),
        5 => (128, 0, 128),
        6 => (0, 128, 128),
        7 => (192, 192, 192),
        8 => (128, 128, 128),
        9 => (255, 0, 0),
        10 => (0, 255, 0),
        11 => (255, 255, 0),
        12 => (0, 0, 255),
        13 => (255, 0, 255),
        14 => (0, 255, 255),
        15 => (255, 255, 255),
        16..=231 => {
            let n = i - 16;
            let r = n / 36;
            let g = (n / 6) % 6;
            let b = n % 6;
            let q = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
            (q(r), q(g), q(b))
        }
        232..=255 => {
            let v = 8 + (i - 232) * 10;
            (v, v, v)
        }
    }
}

fn parse_rgba_color(s: &str) -> Option<GraphColor> {
    if !s.starts_with('#') {
        return None;
    }

    let s = &s[1..];
    let l = s.len();
    if l != 6 && l != 8 {
        return None;
    }

    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    if l == 6 {
        Some(GraphColor::from_rgb(r, g, b))
    } else {
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(GraphColor::from_rgba(r, g, b, a))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("#ff0000", Some(GraphColor { r: 255, g: 0, b: 0, a: 255}))]
    #[case("#AABBCCDD", Some(GraphColor { r: 170, g: 187, b: 204, a: 221}))]
    #[case("#ff000", None)]
    #[case("#fff", None)]
    #[case("000000", None)]
    #[case("##123456", None)]
    fn test_parse_rgba_color(#[case] input: &str, #[case] expected: Option<GraphColor>) {
        assert_eq!(parse_rgba_color(input), expected);
    }

    #[test]
    fn padding_bg_lighter_lifts_reset_and_black() {
        assert_eq!(
            padding_bg(RatatuiColor::Reset, PaddingShade::Lighter),
            RatatuiColor::Rgb(96, 96, 96)
        );
        assert_eq!(
            padding_bg(RatatuiColor::Black, PaddingShade::Lighter),
            RatatuiColor::Rgb(96, 96, 96)
        );
    }

    #[test]
    fn padding_bg_darker_reset_is_black() {
        assert_eq!(
            padding_bg(RatatuiColor::Reset, PaddingShade::Darker),
            RatatuiColor::Black
        );
    }

    #[test]
    fn padding_bg_shifts_rgb_both_ways() {
        let src = RatatuiColor::Rgb(16, 16, 24);
        assert_eq!(
            padding_bg(src, PaddingShade::Lighter),
            RatatuiColor::Rgb(112, 112, 120)
        );
        assert_eq!(
            padding_bg(src, PaddingShade::Darker),
            RatatuiColor::Rgb(0, 0, 0)
        );
        assert_eq!(
            padding_bg(RatatuiColor::Rgb(240, 240, 240), PaddingShade::Darker),
            RatatuiColor::Rgb(144, 144, 144)
        );
        assert_eq!(
            padding_bg(RatatuiColor::Rgb(240, 240, 240), PaddingShade::Lighter),
            RatatuiColor::Rgb(255, 255, 255)
        );
    }
}
