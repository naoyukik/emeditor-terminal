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

    /// Solarized Dark
    pub const fn solarized_dark() -> Self {
        Self {
            default_fg: RgbColor::new(131, 148, 150), // base0
            default_bg: RgbColor::new(0, 43, 54),     // base03
            ansi_palette: [
                RgbColor::new(7, 54, 66),     // Black (base02)
                RgbColor::new(220, 50, 47),   // Red
                RgbColor::new(133, 153, 0),   // Green
                RgbColor::new(181, 137, 0),   // Yellow
                RgbColor::new(38, 139, 210),  // Blue
                RgbColor::new(211, 54, 130),  // Magenta
                RgbColor::new(42, 161, 152),  // Cyan
                RgbColor::new(238, 232, 213), // White (base2)
                RgbColor::new(0, 43, 54),     // Bright Black (base03)
                RgbColor::new(203, 75, 22),   // Bright Red (orange)
                RgbColor::new(88, 110, 117),  // Bright Green (base01)
                RgbColor::new(101, 123, 131), // Bright Yellow (base00)
                RgbColor::new(131, 148, 150), // Bright Blue (base0)
                RgbColor::new(108, 113, 196), // Bright Magenta (violet)
                RgbColor::new(147, 161, 161), // Bright Cyan (base1)
                RgbColor::new(253, 246, 227), // Bright White (base3)
            ],
        }
    }

    /// Solarized Light
    pub const fn solarized_light() -> Self {
        Self {
            default_fg: RgbColor::new(101, 123, 131), // base00
            default_bg: RgbColor::new(253, 246, 227), // base3
            ansi_palette: [
                RgbColor::new(238, 232, 213), // Black (base2)
                RgbColor::new(220, 50, 47),   // Red
                RgbColor::new(133, 153, 0),   // Green
                RgbColor::new(181, 137, 0),   // Yellow
                RgbColor::new(38, 139, 210),  // Blue
                RgbColor::new(211, 54, 130),  // Magenta
                RgbColor::new(42, 161, 152),  // Cyan
                RgbColor::new(7, 54, 66),     // White (base02)
                RgbColor::new(253, 246, 227), // Bright Black (base3)
                RgbColor::new(203, 75, 22),   // Bright Red (orange)
                RgbColor::new(147, 161, 161), // Bright Green (base1)
                RgbColor::new(131, 148, 150), // Bright Yellow (base0)
                RgbColor::new(101, 123, 131), // Bright Blue (base00)
                RgbColor::new(108, 113, 196), // Bright Magenta (violet)
                RgbColor::new(88, 110, 117),  // Bright Cyan (base01)
                RgbColor::new(0, 43, 54),     // Bright White (base03)
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
    fn test_solarized_dark_colors() {
        let theme = ColorTheme::solarized_dark();
        assert_eq!(theme.default_bg, RgbColor::new(0, 43, 54));
        assert_eq!(theme.default_fg, RgbColor::new(131, 148, 150));
        assert_eq!(theme.ansi_palette[0], RgbColor::new(7, 54, 66));
    }
}
