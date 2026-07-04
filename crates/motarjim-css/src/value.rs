/// A parsed CSS length with its unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CssLength {
    /// Pixels (`px`).
    Px(f64),
    /// Ems (`em`) — relative to parent font-size.
    Em(f64),
    /// Rems (`rem`) — relative to root font-size.
    Rem(f64),
    /// Percentage (`%`).
    Percent(f64),
    /// Viewport width (`vw`).
    Vw(f64),
    /// Viewport height (`vh`).
    Vh(f64),
    /// A raw numeric value without explicit unit (for unitless properties).
    Raw(f64),
}

/// A parsed CSS color value.
#[derive(Debug, Clone, PartialEq)]
pub enum CssColor {
    /// Hex color e.g. `#ff0000`.
    Hex(u8, u8, u8, u8),
    /// Named color.
    Named(String),
    /// RGB / RGBA functional notation.
    Rgba(u8, u8, u8, f64),
    /// The `transparent` keyword.
    Transparent,
    /// The `currentColor` keyword.
    CurrentColor,
    /// An unrecognised colour string (kept as-is).
    Other(String),
}

/// A parsed CSS value (broad enough for our engine).
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    /// A keyword (e.g. `auto`, `inherit`, `none`).
    Keyword(String),
    /// A length.
    Length(CssLength),
    /// A colour.
    Color(CssColor),
    /// A percentage.
    Percentage(f64),
    /// A plain number.
    Number(f64),
    /// A raw string (unrecognised).
    Raw(String),
}

// ---------------------------------------------------------------------------
// Value parsing helpers
// ---------------------------------------------------------------------------

/// Try to parse a CSS colour from a raw string value.
///
/// Supports: named colours, `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()`.
#[must_use]
pub fn parse_color(raw: &str) -> Option<CssColor> {
    let raw = raw.trim().to_lowercase();

    // Named colours (subset of CSS named colours)
    if let Some(c) = named_color(&raw) {
        return Some(CssColor::Rgba(c.0, c.1, c.2, 1.0));
    }

    if raw == "transparent" {
        return Some(CssColor::Transparent);
    }
    if raw == "currentcolor" {
        return Some(CssColor::CurrentColor);
    }

    // Hex: #rgb / #rrggbb / #rrggbbaa
    if let Some(hex) = raw.strip_prefix('#') {
        return parse_hex_color(hex);
    }

    // rgb() / rgba()
    if raw.starts_with("rgb") {
        return parse_rgb_function(&raw);
    }

    None
}

/// Returns the RGB values for a named CSS color, if recognized.
fn named_color(name: &str) -> Option<(u8, u8, u8)> {
    Some(match name {
        "black" => (0, 0, 0),
        "silver" => (192, 192, 192),
        "gray" | "grey" => (128, 128, 128),
        "white" => (255, 255, 255),
        "maroon" => (128, 0, 0),
        "red" => (255, 0, 0),
        "purple" => (128, 0, 128),
        "fuchsia" => (255, 0, 255),
        "green" => (0, 128, 0),
        "lime" => (0, 255, 0),
        "olive" => (128, 128, 0),
        "yellow" => (255, 255, 0),
        "navy" => (0, 0, 128),
        "blue" => (0, 0, 255),
        "teal" => (0, 128, 128),
        "aqua" => (0, 255, 255),
        "orange" => (255, 165, 0),
        "pink" => (255, 192, 203),
        "coral" => (255, 127, 80),
        "tomato" => (255, 99, 71),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "darkred" => (139, 0, 0),
        "darkgreen" => (0, 100, 0),
        "darkblue" => (0, 0, 139),
        "darkorange" => (255, 140, 0),
        "darkviolet" => (148, 0, 211),
        "gold" => (255, 215, 0),
        "brown" => (165, 42, 42),
        "crimson" => (220, 20, 60),
        "indigo" => (75, 0, 130),
        "khaki" => (240, 230, 140),
        "lavender" => (230, 230, 250),
        "linen" => (250, 240, 230),
        "magenta" => (255, 0, 255),
        "mintcream" => (245, 255, 250),
        "navajowhite" => (255, 222, 173),
        "oldlace" => (253, 245, 230),
        "plum" => (221, 160, 221),
        "salmon" => (250, 128, 114),
        "seagreen" => (46, 139, 87),
        "sienna" => (160, 82, 45),
        "slategray" | "slategrey" => (112, 128, 144),
        "steelblue" => (70, 130, 180),
        "aliceblue" => (240, 248, 255),
        "azure" => (240, 255, 255),
        "beige" => (245, 245, 220),
        "bisque" => (255, 228, 196),
        "blanchedalmond" => (255, 235, 205),
        "burlywood" => (222, 184, 135),
        "cadetblue" => (95, 158, 160),
        "chocolate" => (210, 105, 30),
        "cornflowerblue" => (100, 149, 237),
        "cornsilk" => (255, 248, 220),
        "cyan" => (0, 255, 255),
        "deeppink" => (255, 20, 147),
        "deepskyblue" => (0, 191, 255),
        "dimgray" | "dimgrey" => (105, 105, 105),
        "dodgerblue" => (30, 144, 255),
        "firebrick" => (178, 34, 34),
        "floralwhite" => (255, 250, 240),
        "forestgreen" => (34, 139, 34),
        "gainsboro" => (220, 220, 220),
        "ghostwhite" => (248, 248, 255),
        "honeydew" => (240, 255, 240),
        "hotpink" => (255, 105, 180),
        "ivory" => (255, 255, 240),
        "lace" => (253, 245, 230),
        "lemonchiffon" => (255, 250, 205),
        "lightblue" => (173, 216, 230),
        "lightcoral" => (240, 128, 128),
        "lightcyan" => (224, 255, 255),
        "lightgoldenrodyellow" => (250, 250, 210),
        "lightgreen" => (144, 238, 144),
        "lightpink" => (255, 182, 193),
        "lightsalmon" => (255, 160, 122),
        "lightseagreen" => (32, 178, 170),
        "lightskyblue" => (135, 206, 250),
        "lightslategray" | "lightslategrey" => (119, 136, 153),
        "lightsteelblue" => (176, 196, 222),
        "lightyellow" => (255, 255, 224),
        "limegreen" => (50, 205, 50),
        "mediumaquamarine" => (102, 205, 170),
        "mediumblue" => (0, 0, 205),
        "mediumorchid" => (186, 85, 211),
        "mediumpurple" => (147, 112, 219),
        "mediumseagreen" => (60, 179, 113),
        "mediumslateblue" => (123, 104, 238),
        "mediumspringgreen" => (0, 250, 154),
        "mediumturquoise" => (72, 209, 204),
        "mediumvioletred" => (199, 21, 133),
        "midnightblue" => (25, 25, 112),
        "mistyrose" => (255, 228, 225),
        "moccasin" => (255, 228, 181),
        "oldgold" => (207, 181, 59),
        "olivedrab" => (107, 142, 35),
        "orangered" => (255, 69, 0),
        "orchid" => (218, 112, 214),
        "palegoldenrod" => (238, 232, 170),
        "palegreen" => (152, 251, 152),
        "paleturquoise" => (175, 238, 238),
        "palevioletred" => (219, 112, 147),
        "papayawhip" => (255, 239, 213),
        "peachpuff" => (255, 218, 185),
        "peru" => (205, 133, 63),
        "powderblue" => (176, 224, 230),
        "rebeccapurple" => (102, 51, 153),
        "rosybrown" => (188, 143, 143),
        "royalblue" => (65, 105, 225),
        "saddlebrown" => (139, 69, 19),
        "sandybrown" => (244, 164, 96),
        "seashell" => (255, 245, 238),
        "skyblue" => (135, 206, 235),
        "slateblue" => (106, 90, 205),
        "snow" => (255, 250, 250),
        "springgreen" => (0, 255, 127),
        "yellowgreen" => (154, 205, 50),
        _ => return None,
    })
}

/// Parses a hex color string (e.g. `#ff0000`) into a `CssColor`.
fn parse_hex_color(hex: &str) -> Option<CssColor> {
    let digits: String = hex.chars().filter(char::is_ascii_hexdigit).collect();
    match digits.len() {
        3 => {
            let r = u8::from_str_radix(&format!("{}{}", &digits[0..1], &digits[0..1]), 16).ok()?;
            let g = u8::from_str_radix(&format!("{}{}", &digits[1..2], &digits[1..2]), 16).ok()?;
            let b = u8::from_str_radix(&format!("{}{}", &digits[2..3], &digits[2..3]), 16).ok()?;
            Some(CssColor::Hex(r, g, b, 255))
        }
        4 => {
            let r = u8::from_str_radix(&format!("{}{}", &digits[0..1], &digits[0..1]), 16).ok()?;
            let g = u8::from_str_radix(&format!("{}{}", &digits[1..2], &digits[1..2]), 16).ok()?;
            let b = u8::from_str_radix(&format!("{}{}", &digits[2..3], &digits[2..3]), 16).ok()?;
            let a = u8::from_str_radix(&format!("{}{}", &digits[3..4], &digits[3..4]), 16).ok()?;
            Some(CssColor::Hex(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&digits[0..2], 16).ok()?;
            let g = u8::from_str_radix(&digits[2..4], 16).ok()?;
            let b = u8::from_str_radix(&digits[4..6], 16).ok()?;
            Some(CssColor::Hex(r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&digits[0..2], 16).ok()?;
            let g = u8::from_str_radix(&digits[2..4], 16).ok()?;
            let b = u8::from_str_radix(&digits[4..6], 16).ok()?;
            let a = u8::from_str_radix(&digits[6..8], 16).ok()?;
            Some(CssColor::Hex(r, g, b, a))
        }
        _ => None,
    }
}

/// Parses an `rgb()` or `rgba()` function string into a `CssColor`.
fn parse_rgb_function(raw: &str) -> Option<CssColor> {
    let inner = raw
        .trim_start_matches("rgba")
        .trim_start_matches("rgb")
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();

    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();

    let r = parts.first()?.parse::<u8>().ok()?;
    let g = parts.get(1)?.parse::<u8>().ok()?;
    let b = parts.get(2)?.parse::<u8>().ok()?;

    let a = parts
        .get(3)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0);

    Some(CssColor::Rgba(r, g, b, a.clamp(0.0, 1.0)))
}

/// Parse a CSS length value (e.g. `"10px"`, `"1.5em"`, `"50%"`, `"auto"`).
#[must_use]
pub fn parse_length(raw: &str) -> Option<CssLength> {
    let raw = raw.trim().to_lowercase();

    if raw == "auto" || raw == "inherit" || raw == "initial" || raw == "unset" {
        return None;
    }

    // Check for percentage
    if let Some(num) = raw.strip_suffix('%') {
        let v = num.trim().parse::<f64>().ok()?;
        return Some(CssLength::Percent(v));
    }

    // Find the boundary between number and unit
    let num_end = raw
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
        .unwrap_or(raw.len());

    if num_end == 0 {
        return None;
    }

    let num_str = &raw[..num_end];
    let unit = &raw[num_end..];

    let value = num_str.parse::<f64>().ok()?;

    match unit {
        "px" => Some(CssLength::Px(value)),
        "em" => Some(CssLength::Em(value)),
        "rem" => Some(CssLength::Rem(value)),
        "vw" => Some(CssLength::Vw(value)),
        "vh" => Some(CssLength::Vh(value)),
        "" => Some(CssLength::Raw(value)),
        _ => None,
    }
}

/// Parse a numeric CSS value (e.g. `"42"`, `"3.14"`, `"0"`).
#[must_use]
pub fn parse_number(raw: &str) -> Option<f64> {
    raw.trim().parse::<f64>().ok()
}

// ---------------------------------------------------------------------------
// Resolved declaration
// ---------------------------------------------------------------------------
