use crate::config::GraphColorConfig;

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

    pub fn to_ratatui_color(self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
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
}
