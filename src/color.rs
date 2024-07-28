#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }

    pub fn to_image_color(self) -> image::Rgba<u8> {
        image::Rgba([self.r, self.g, self.b, self.a])
    }

    pub fn to_ratatui_color(self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone)]
pub struct ColorSet {
    pub colors: Vec<Color>,
}

impl ColorSet {
    pub fn get(&self, index: usize) -> Color {
        self.colors[index % self.colors.len()]
    }
}

impl Default for ColorSet {
    fn default() -> Self {
        Self {
            colors: vec![
                Color::from_rgb(224, 108, 118),
                Color::from_rgb(152, 195, 121),
                Color::from_rgb(229, 192, 123),
                Color::from_rgb(97, 175, 239),
                Color::from_rgb(198, 120, 221),
                Color::from_rgb(86, 182, 194),
            ],
        }
    }
}
