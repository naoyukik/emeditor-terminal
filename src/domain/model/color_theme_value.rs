#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorTheme {
    pub default_fg: RgbColor,
    pub default_bg: RgbColor,
    pub ansi_palette: [RgbColor; 16],
}

impl ColorTheme {
    /// Campbell (Windows Terminal Default)
    pub const fn campbell() -> Self {
        Self {
            default_fg: RgbColor::new(204, 204, 204),
            default_bg: RgbColor::new(12, 12, 12),
            ansi_palette: [
                RgbColor::new(12, 12, 12),    // Black
                RgbColor::new(197, 15, 31),   // Red
                RgbColor::new(19, 161, 14),   // Green
                RgbColor::new(193, 156, 0),   // Yellow
                RgbColor::new(0, 55, 218),    // Blue
                RgbColor::new(136, 23, 152),  // Magenta
                RgbColor::new(58, 150, 221),  // Cyan
                RgbColor::new(204, 204, 204), // White
                RgbColor::new(118, 118, 118), // Bright Black
                RgbColor::new(231, 72, 86),   // Bright Red
                RgbColor::new(22, 198, 12),   // Bright Green
                RgbColor::new(249, 241, 165), // Bright Yellow
                RgbColor::new(59, 120, 255),  // Bright Blue
                RgbColor::new(180, 0, 158),   // Bright Magenta
                RgbColor::new(97, 214, 214),  // Bright Cyan
                RgbColor::new(242, 242, 242), // Bright White
            ],
        }
    }

    /// One Half Dark
    pub const fn one_half_dark() -> Self {
        Self {
            default_fg: RgbColor::new(220, 223, 228), // #dcdfe4
            default_bg: RgbColor::new(40, 44, 52),    // #282c34
            ansi_palette: [
                RgbColor::new(56, 58, 66),    // Black #383a42
                RgbColor::new(224, 108, 117), // Red #e06c75
                RgbColor::new(152, 195, 121), // Green #98c379
                RgbColor::new(229, 192, 123), // Yellow #e5c07b
                RgbColor::new(97, 175, 239),  // Blue #61afef
                RgbColor::new(198, 120, 221), // Magenta #c678dd
                RgbColor::new(86, 182, 194),  // Cyan #56b6c2
                RgbColor::new(220, 223, 228), // White #dcdfe4
                RgbColor::new(127, 132, 142), // Bright Black #7f848e
                RgbColor::new(224, 108, 117), // Bright Red #e06c75
                RgbColor::new(152, 195, 121), // Bright Green #98c379
                RgbColor::new(229, 192, 123), // Bright Yellow #e5c07b
                RgbColor::new(97, 175, 239),  // Bright Blue #61afef
                RgbColor::new(198, 120, 221), // Bright Purple #c678dd
                RgbColor::new(86, 182, 194),  // Bright Cyan #56b6c2
                RgbColor::new(255, 255, 255), // Bright White #ffffff
            ],
        }
    }

    /// One Half Light
    pub const fn one_half_light() -> Self {
        Self {
            default_fg: RgbColor::new(56, 58, 66),    // #383a42
            default_bg: RgbColor::new(250, 250, 250), // #fafafa
            ansi_palette: [
                RgbColor::new(56, 58, 66),    // Black #383a42
                RgbColor::new(228, 86, 73),   // Red #e45649
                RgbColor::new(80, 161, 79),   // Green #50a14f
                RgbColor::new(193, 132, 1),   // Yellow #c18401
                RgbColor::new(1, 132, 188),   // Blue #0184bc
                RgbColor::new(166, 38, 164),  // Magenta #a626a4
                RgbColor::new(9, 151, 179),   // Cyan #0997b3
                RgbColor::new(250, 250, 250), // White #fafafa
                RgbColor::new(160, 161, 167), // Bright Black #a0a1a7
                RgbColor::new(228, 86, 73),   // Bright Red #e45649
                RgbColor::new(80, 161, 79),   // Bright Green #50a14f
                RgbColor::new(193, 132, 1),   // Bright Yellow #c18401
                RgbColor::new(1, 132, 188),   // Bright Blue #0184bc
                RgbColor::new(166, 38, 164),  // Bright Purple #a626a4
                RgbColor::new(9, 151, 179),   // Bright Cyan #0997b3
                RgbColor::new(250, 250, 250), // Bright White #fafafa
            ],
        }
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self::campbell()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_half_dark_colors() {
        let theme = ColorTheme::one_half_dark();
        assert_eq!(theme.default_bg, RgbColor::new(40, 44, 52));
        assert_eq!(theme.default_fg, RgbColor::new(220, 223, 228));
        assert_eq!(theme.ansi_palette[0], RgbColor::new(56, 58, 66));
        assert_eq!(theme.ansi_palette[8], RgbColor::new(127, 132, 142));
    }
}
