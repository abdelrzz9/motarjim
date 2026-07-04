#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! CSS engine for the Motarjim compiler.
//!
//! Provides cascade resolution, computed style calculation, CSS value parsing,
//! and parallel selector matching.
//!
//! # Overview
//!
//! The engine is built around three core types:
//!
//! - [`Cascade`] — Collects and sorts declarations by specificity and source order.
//! - [`ComputedValues`] — Holds the final resolved [`ComputedStyle`] for an element.
//! - [`StyleResolver`] — Accepts stylesheets, matches selectors, computes styles.
//!
//! Selector matching against DOM elements is parallelised via `rayon` when
//! [`StyleResolver::resolve_parallel`] is used.

use std::collections::HashMap;

use motarjim_ast::css::{CssRule, CssStylesheet, Declaration, StyleRule};
use motarjim_ast::selector::{AttributeOperator, Selector, SimpleSelector};
use motarjim_ast::Element;
use motarjim_ast::style::{
    AlignContent, AlignItems, Background, Border, ComputedStyle, DisplayType, EdgeValues,
    FlexDirection, FlexWrap, FontWeight, JustifyContent, Overflow, PositionType, TextAlign,
};
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// CSS value types
// ---------------------------------------------------------------------------

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

    let parts: Vec<&str> = inner
        .split(',')
        .map(str::trim)
        .collect();

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
    let num_end = raw.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
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

/// A single declaration after cascade resolution, carrying its specificity and
/// source order for tie-breaking.
#[derive(Debug, Clone)]
pub struct ResolvedDeclaration {
    /// The CSS property name.
    pub property: SmolStr,
    /// The raw CSS value string.
    pub value: String,
    /// Whether this declaration has `!important`.
    pub important: bool,
    /// Specificity of the selector that matched this declaration `(id, class, type)`.
    pub specificity: (u32, u32, u32),
    /// Source order index (lower = earlier in source).
    pub source_order: usize,
}

// ---------------------------------------------------------------------------
// Cascade
// ---------------------------------------------------------------------------

/// Collects declarations that match an element and sorts them by CSS cascade
/// rules (specificity, importance, source order).
#[derive(Debug, Clone)]
pub struct Cascade {
    /// Resolved declarations sorted by cascade order.
    declarations: Vec<ResolvedDeclaration>,
    /// Next insertion order counter.
    next_order: usize,
}

impl Cascade {
    /// Create an empty cascade.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            declarations: Vec::new(),
            next_order: 0,
        }
    }

    /// Push a batch of declarations from a matched rule.
    pub fn add_declarations(
        &mut self,
        declarations: &[Declaration],
        specificity: (u32, u32, u32),
    ) {
        for decl in declarations {
            self.declarations.push(ResolvedDeclaration {
                property: decl.property.clone(),
                value: decl.value.clone(),
                important: decl.important,
                specificity,
                source_order: self.next_order,
            });
            self.next_order += 1;
        }
    }

    /// Resolve the cascade: sort declarations by importance, specificity, and
    /// source order, keeping only the winning value per property.
    ///
    /// Returns a map of property name → resolved value.
    #[must_use]
    pub fn resolve(&self) -> HashMap<SmolStr, String> {
        let mut sorted = self.declarations.clone();

        // Sort: !important first (higher priority), then specificity (higher wins),
        // then source order (later wins).
        //
        // We sort in ascending order and then keep the *last* occurrence per property
        // since later entries override earlier ones at the same specificity level.
        sorted.sort_by(|a, b| {
            // !important wins over non-important
            let imp_cmp = a.important.cmp(&b.important);
            if imp_cmp != std::cmp::Ordering::Equal {
                return imp_cmp;
            }
            // Higher specificity wins
            let spec_cmp = (
                a.specificity.0,
                a.specificity.1,
                a.specificity.2,
            )
            .cmp(&(
                b.specificity.0,
                b.specificity.1,
                b.specificity.2,
            ));
            if spec_cmp != std::cmp::Ordering::Equal {
                return spec_cmp;
            }
            // Later source order wins
            a.source_order.cmp(&b.source_order)
        });

        // Keep the last (winning) declaration for each property.
        let mut result: HashMap<SmolStr, String> = HashMap::new();
        for decl in &sorted {
            result.insert(decl.property.clone(), decl.value.clone());
        }

        result
    }
}

impl Default for Cascade {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ComputedValues
// ---------------------------------------------------------------------------

/// Wraps a [`ComputedStyle`] with convenient construction helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedValues {
    /// The underlying computed style.
    pub style: ComputedStyle,
}

impl ComputedValues {
    /// Create a new computed values wrapper with default style.
    #[must_use]
    pub fn new() -> Self {
        Self {
            style: ComputedStyle::default(),
        }
    }

    /// Build computed values from a map of resolved declarations and an optional
    /// parent style (for inheritance).
    #[must_use]
    pub fn from_map(
        map: &HashMap<SmolStr, String>,
        parent: Option<&Self>,
    ) -> Self {
        let mut cv = if let Some(p) = parent {
            // Inherit from parent first
            Self {
                style: p.style.clone(),
            }
        } else {
            Self::new()
        };

        // Apply resolved declarations on top (or as initial values).
        apply_declarations(&mut cv.style, map);
        cv
    }
}

impl Default for ComputedValues {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ComputedStyle> for ComputedValues {
    fn from(style: ComputedStyle) -> Self {
        Self { style }
    }
}

// ---------------------------------------------------------------------------
// Property application
// ---------------------------------------------------------------------------

/// Apply a map of resolved declarations onto a `ComputedStyle`.
fn apply_declarations(style: &mut ComputedStyle, map: &HashMap<SmolStr, String>) {
    for (prop, value) in map {
        apply_property(style, prop.as_str(), value);
    }
}

/// Apply a single CSS property-value pair to a `ComputedStyle`.
#[allow(clippy::too_many_lines)]
fn apply_property(style: &mut ComputedStyle, property: &str, value: &str) {
    let v = value.trim();
    if v.is_empty() || v == "inherit" || v == "initial" || v == "unset" {
        return;
    }

    match property {
        "display" => {
            style.display = match v {
                "block" => DisplayType::Block,
                "inline" => DisplayType::Inline,
                "inline-block" => DisplayType::InlineBlock,
                "flex" | "inline-flex" => DisplayType::Flex,
                "grid" | "inline-grid" => DisplayType::Grid,
                "none" => DisplayType::None,
                "contents" => DisplayType::Contents,
                "flow" => DisplayType::Flow,
                "flow-root" => DisplayType::FlowRoot,
                "table" => DisplayType::Table,
                "table-row" => DisplayType::TableRow,
                "table-cell" => DisplayType::TableCell,
                "list-item" => DisplayType::ListItem,
                _ => DisplayType::Block,
            };
        }
        "position" => {
            style.position = match v {
                "static" => PositionType::Static,
                "relative" => PositionType::Relative,
                "absolute" => PositionType::Absolute,
                "fixed" => PositionType::Fixed,
                "sticky" => PositionType::Sticky,
                _ => PositionType::Static,
            };
        }
        "width" | "height" | "min-width" | "min-height" | "max-width" | "max-height"
            if v != "auto" => {
                set_dimension(style, property, v);
            }
        "margin" => {
            set_edge_values_shorthand(style, v, |s| &mut s.margin);
        }
        "margin-top" => {
            if let Some(l) = parse_length(v) {
                style.margin.top = length_to_px(l, 16.0);
            }
        }
        "margin-right" => {
            if let Some(l) = parse_length(v) {
                style.margin.right = length_to_px(l, 16.0);
            }
        }
        "margin-bottom" => {
            if let Some(l) = parse_length(v) {
                style.margin.bottom = length_to_px(l, 16.0);
            }
        }
        "margin-left" => {
            if let Some(l) = parse_length(v) {
                style.margin.left = length_to_px(l, 16.0);
            }
        }
        "padding" => {
            set_edge_values_shorthand(style, v, |s| &mut s.padding);
        }
        "padding-top" => {
            if let Some(l) = parse_length(v) {
                style.padding.top = length_to_px(l, 16.0);
            }
        }
        "padding-right" => {
            if let Some(l) = parse_length(v) {
                style.padding.right = length_to_px(l, 16.0);
            }
        }
        "padding-bottom" => {
            if let Some(l) = parse_length(v) {
                style.padding.bottom = length_to_px(l, 16.0);
            }
        }
        "padding-left" => {
            if let Some(l) = parse_length(v) {
                style.padding.left = length_to_px(l, 16.0);
            }
        }
        "color" => {
            style.color = Some(v.to_string());
        }
        "background" | "background-color" => {
            let bg = style.background.get_or_insert(Background {
                color: None,
                image: None,
                position: None,
                repeat: None,
                size: None,
            });
            if v != "none" {
                bg.color = Some(v.to_string());
            }
        }
        "background-image" => {
            let bg = style.background.get_or_insert(Background {
                color: None,
                image: None,
                position: None,
                repeat: None,
                size: None,
            });
            bg.image = Some(v.to_string());
        }
        "border" | "border-width" => {
            if let Some(l) = parse_length(v) {
                let border = style.border.get_or_insert_with(|| Border {
                    width: EdgeValues::default(),
                    color: None,
                    style: None,
                    radius: EdgeValues::default(),
                });
                let px = length_to_px(l, 16.0);
                border.width = EdgeValues::all(px);
            }
        }
        "border-color" => {
            let border = style.border.get_or_insert_with(|| Border {
                width: EdgeValues::default(),
                color: None,
                style: None,
                radius: EdgeValues::default(),
            });
            border.color = Some(v.to_string());
        }
        "border-radius" => {
            if let Some(l) = parse_length(v) {
                let border = style.border.get_or_insert_with(|| Border {
                    width: EdgeValues::default(),
                    color: None,
                    style: None,
                    radius: EdgeValues::default(),
                });
                let px = length_to_px(l, 16.0);
                border.radius = EdgeValues::all(px);
            }
        }
        "flex-direction" => {
            style.flex_direction = match v {
                "row" => Some(FlexDirection::Row),
                "column" => Some(FlexDirection::Column),
                "row-reverse" => Some(FlexDirection::RowReverse),
                "column-reverse" => Some(FlexDirection::ColumnReverse),
                _ => None,
            };
        }
        "flex-wrap" => {
            style.flex_wrap = match v {
                "nowrap" => Some(FlexWrap::NoWrap),
                "wrap" => Some(FlexWrap::Wrap),
                "wrap-reverse" => Some(FlexWrap::WrapReverse),
                _ => None,
            };
        }
        "flex-grow" => {
            style.flex_grow = parse_number(v).unwrap_or(0.0);
        }
        "flex-shrink" => {
            style.flex_shrink = parse_number(v).unwrap_or(1.0);
        }
        "flex-basis"
            if v != "auto" => {
                style.flex_basis = Some(v.to_string());
            }
        "flex" => {
            // Simple flex shorthand: parse 1-3 values
            let parts: Vec<&str> = v.split_whitespace().collect();
            if let Some(first) = parts.first() {
                if let Some(n) = parse_number(first) {
                    if parts.len() == 1 {
                        style.flex_grow = n;
                        style.flex_shrink = 1.0;
                    } else if parts.len() >= 2 {
                        style.flex_grow = n;
                        if let Some(s) = parts.get(1).and_then(|p| parse_number(p)) {
                            style.flex_shrink = s;
                        }
                        if let Some(basis) = parts.get(2) {
                            style.flex_basis = Some(basis.to_string());
                        }
                    }
                } else {
                    style.flex_basis = Some(first.to_string());
                    if let Some(s) = parts.get(1).and_then(|p| parse_number(p)) {
                        style.flex_shrink = s;
                    }
                }
            }
        }
        "justify-content" => {
            style.justify_content = match v {
                "flex-start" | "start" => Some(JustifyContent::FlexStart),
                "flex-end" | "end" => Some(JustifyContent::FlexEnd),
                "center" => Some(JustifyContent::Center),
                "space-between" => Some(JustifyContent::SpaceBetween),
                "space-around" => Some(JustifyContent::SpaceAround),
                "space-evenly" => Some(JustifyContent::SpaceEvenly),
                _ => None,
            };
        }
        "align-items" => {
            style.align_items = match v {
                "flex-start" | "start" => Some(AlignItems::FlexStart),
                "flex-end" | "end" => Some(AlignItems::FlexEnd),
                "center" => Some(AlignItems::Center),
                "stretch" => Some(AlignItems::Stretch),
                "baseline" => Some(AlignItems::Baseline),
                _ => None,
            };
        }
        "align-content" => {
            style.align_content = match v {
                "flex-start" | "start" => Some(AlignContent::FlexStart),
                "flex-end" | "end" => Some(AlignContent::FlexEnd),
                "center" => Some(AlignContent::Center),
                "stretch" => Some(AlignContent::Stretch),
                "space-between" => Some(AlignContent::SpaceBetween),
                "space-around" => Some(AlignContent::SpaceAround),
                _ => None,
            };
        }
        "align-self" => {
            style.align_self = match v {
                "flex-start" | "start" => Some(AlignItems::FlexStart),
                "flex-end" | "end" => Some(AlignItems::FlexEnd),
                "center" => Some(AlignItems::Center),
                "stretch" => Some(AlignItems::Stretch),
                "baseline" => Some(AlignItems::Baseline),
                "auto" => None,
                _ => None,
            };
        }
        "gap" | "row-gap" | "column-gap"
            if v != "normal" => {
                match property {
                    "gap" => style.gap = Some(v.to_string()),
                    "row-gap" => style.row_gap = Some(v.to_string()),
                    "column-gap" => style.column_gap = Some(v.to_string()),
                    _ => {}
                }
            }
        "grid-template-columns" => style.grid_template_columns = Some(v.to_string()),
        "grid-template-rows" => style.grid_template_rows = Some(v.to_string()),
        "grid-column" => style.grid_column = Some(v.to_string()),
        "grid-row" => style.grid_row = Some(v.to_string()),
        "font-family" => style.font_family = Some(v.to_string()),
        "font-size" => style.font_size = Some(v.to_string()),
        "font-weight" => {
            style.font_weight = parse_font_weight(v);
        }
        "font-style" => style.font_style = Some(v.to_string()),
        "line-height" => style.line_height = Some(v.to_string()),
        "text-align" => {
            style.text_align = match v {
                "left" => Some(TextAlign::Left),
                "right" => Some(TextAlign::Right),
                "center" => Some(TextAlign::Center),
                "justify" => Some(TextAlign::Justify),
                _ => None,
            };
        }
        "text-decoration" => style.text_decoration = Some(v.to_string()),
        "opacity" => {
            if let Some(n) = parse_number(v) {
                style.opacity = n.clamp(0.0, 1.0);
            }
        }
        "overflow" => {
            style.overflow = match v {
                "visible" => Some(Overflow::Visible),
                "hidden" => Some(Overflow::Hidden),
                "scroll" => Some(Overflow::Scroll),
                "auto" => Some(Overflow::Auto),
                _ => None,
            };
        }
        "cursor" => style.cursor = Some(v.to_string()),
        "box-shadow" => style.box_shadow = Some(v.to_string()),
        "transform" => style.transform = Some(v.to_string()),
        "transition" => style.transition = Some(v.to_string()),
        "visibility" => {
            style.visibility = v != "hidden" && v != "collapse";
        }
        "z-index" => {
            if let Ok(n) = v.parse::<i32>() {
                style.z_index = Some(n);
            }
        }
        "pointer-events" => style.pointer_events = Some(v.to_string()),
        "resize" => style.resize = Some(v.to_string()),
        "user-select" => style.user_select = Some(v.to_string()),
        "appearance" => style.appearance = Some(v.to_string()),
        _ => {}
    }
}

/// Parse a font-weight string value.
fn parse_font_weight(v: &str) -> Option<FontWeight> {
    Some(match v {
        "thin" | "100" => FontWeight::Thin,
        "extra-light" | "200" | "extralight" => FontWeight::ExtraLight,
        "light" | "300" => FontWeight::Light,
        "normal" | "400" => FontWeight::Normal,
        "medium" | "500" => FontWeight::Medium,
        "semi-bold" | "600" | "semibold" => FontWeight::SemiBold,
        "bold" | "700" => FontWeight::Bold,
        "extra-bold" | "800" | "extrabold" => FontWeight::ExtraBold,
        "black" | "900" => FontWeight::Black,
        _ => {
            if let Ok(n) = v.parse::<u16>() {
                FontWeight::Custom(n.clamp(100, 900))
            } else {
                return None;
            }
        }
    })
}

/// Set a dimension property (width, height, etc.) from a CSS value.
fn set_dimension(style: &mut ComputedStyle, property: &str, value: &str) {
    let v = value.to_string();
    match property {
        "width" => style.width = Some(v),
        "height" => style.height = Some(v),
        "min-width" => style.min_width = Some(v),
        "min-height" => style.min_height = Some(v),
        "max-width" => style.max_width = Some(v),
        "max-height" => style.max_height = Some(v),
        _ => {}
    }
}

/// Apply a CSS shorthand for top/right/bottom/left values (margin, padding).
fn set_edge_values_shorthand(
    style: &mut ComputedStyle,
    value: &str,
    target: fn(&mut ComputedStyle) -> &mut EdgeValues,
) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let lengths: Vec<f64> = parts
        .iter()
        .filter_map(|p| parse_length(p).map(|l| length_to_px(l, 16.0)))
        .collect();

    if lengths.is_empty() {
        return;
    }

    let ev = match lengths.len() {
        1 => EdgeValues::all(lengths[0]),
        2 => EdgeValues::symmetric(lengths[0], lengths[1]),
        3 => EdgeValues::new(lengths[0], lengths[1], lengths[2], lengths[1]),
        4 => EdgeValues::new(lengths[0], lengths[1], lengths[2], lengths[3]),
        _ => return,
    };

    *target(style) = ev;
}

/// Convert a `CssLength` to `f64` pixels (using a default font-size fallback).
#[must_use]
pub fn length_to_px(length: CssLength, font_size: f64) -> f64 {
    match length {
        CssLength::Px(v) => v,
        CssLength::Em(v) => v * font_size,
        CssLength::Rem(v) => v * 16.0,
        CssLength::Percent(v) => v,
        CssLength::Vw(v) => v,
        CssLength::Vh(v) => v,
        CssLength::Raw(v) => v,
    }
}

// ---------------------------------------------------------------------------
// Selector matching helpers
// ---------------------------------------------------------------------------

/// Check whether a rule's selectors match a given element.
fn rule_matches_element(rule: &StyleRule, element: &Element) -> bool {
    rule.selectors.iter().any(|sel| selector_matches_element(sel, element))
}

/// Check whether a single `Selector` matches an element.
fn selector_matches_element(selector: &Selector, element: &Element) -> bool {
    if selector.combinators.is_empty() {
        // Simple compound selector (no combinators): all simple selectors must match.
        return selector
            .simple_selectors
            .iter()
            .all(|s| simple_selector_matches(s, element));
    }

    // For selectors with combinators we do a limited matching — match the last
    // selector group and accept combinators as a simple conjunction.
    // Full combinator-aware matching (walking the DOM tree) is available via the
    // `motarjim-selectors` crate.
    selector
        .simple_selectors
        .iter()
        .all(|s| simple_selector_matches(s, element))
}

/// Check whether a single `SimpleSelector` matches an element.
fn simple_selector_matches(sel: &SimpleSelector, element: &Element) -> bool {
    match sel {
        SimpleSelector::Universal => true,
        SimpleSelector::Type(name) => element.tag_name.as_str() == name.as_str(),
        SimpleSelector::Class(name) => element.has_class(name.as_str()),
        SimpleSelector::Id(name) => element.id.as_ref().is_some_and(|id| id.as_str() == name.as_str()),
        SimpleSelector::Attribute {
            name,
            operator,
            value,
            case_sensitive: _,
        } => {
            let attr_val = match element.get_attribute(name.as_str()) {
                Some(v) => v,
                None => return false,
            };
            match operator {
                None => true,
                Some(AttributeOperator::Equals) => {
                    value.as_ref().is_some_and(|v| attr_val == v.as_str())
                }
                Some(AttributeOperator::Includes) => {
                    value.as_ref().is_some_and(|v| {
                        attr_val.split_whitespace().any(|part| part == v.as_str())
                    })
                }
                Some(AttributeOperator::DashMatch) => value.as_ref().is_some_and(|v| {
                    attr_val == v.as_str()
                        || attr_val.starts_with(&format!("{}-", v.as_str()))
                }),
                Some(AttributeOperator::PrefixMatch) => {
                    value.as_ref().is_some_and(|v| attr_val.starts_with(v.as_str()))
                }
                Some(AttributeOperator::SuffixMatch) => {
                    value.as_ref().is_some_and(|v| attr_val.ends_with(v.as_str()))
                }
                Some(AttributeOperator::SubstringMatch) => {
                    value.as_ref().is_some_and(|v| attr_val.contains(v.as_str()))
                }
            }
        }
        SimpleSelector::PseudoClass(_) | SimpleSelector::PseudoElement(_) => {
            // Pseudo-classes and pseudo-elements are conservatively treated as matching.
            true
        }
    }
}

/// Compute the *maximum* specificity among all selectors in a rule.
fn rule_max_specificity(rule: &StyleRule) -> (u32, u32, u32) {
    rule.selectors
        .iter()
        .map(motarjim_ast::Selector::specificity)
        .max()
        .unwrap_or((0, 0, 0))
}

// ---------------------------------------------------------------------------
// StyleResolver
// ---------------------------------------------------------------------------

/// The main style resolver.
///
/// Accepts parsed stylesheets, matches selectors against DOM elements,
/// resolves the cascade, and computes final styles.
///
/// # Example
///
/// ```rust
/// use motarjim_ast::css::{CssStylesheet, StyleRule, Declaration};
/// use motarjim_ast::Element;
/// use motarjim_css::StyleResolver;
///
/// let mut resolver = StyleResolver::new();
/// resolver.add_stylesheet(CssStylesheet {
///     rules: vec![],
///     source_path: None,
/// });
/// let el = Element::new("div");
/// let values = resolver.resolve(&el);
/// ```
pub struct StyleResolver {
    /// Loaded stylesheets to resolve against.
    stylesheets: Vec<CssStylesheet>,
}

impl StyleResolver {
    /// Create a new empty style resolver.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stylesheets: Vec::new(),
        }
    }

    /// Add a parsed stylesheet to the resolver.
    pub fn add_stylesheet(&mut self, sheet: CssStylesheet) {
        self.stylesheets.push(sheet);
    }

    /// Return a reference to the registered stylesheets.
    #[must_use]
    pub fn stylesheets(&self) -> &[CssStylesheet] {
        &self.stylesheets
    }

    /// Resolve the computed style for a single element.
    ///
    /// Uses the registered stylesheets, matching selectors and resolving the
    /// cascade. If a `parent` value is provided, inheritable properties from
    /// the parent are used as the starting point.
    #[must_use]
    pub fn resolve_with_parent(
        &self,
        element: &Element,
        parent: Option<&ComputedValues>,
    ) -> ComputedValues {
        let mut cascade = Cascade::new();

        for sheet in &self.stylesheets {
            self.collect_matching_declarations(&mut cascade, sheet, element);
        }

        let resolved_map = cascade.resolve();
        ComputedValues::from_map(&resolved_map, parent)
    }

    /// Resolve the computed style for a single element without a parent context.
    #[must_use]
    pub fn resolve(&self, element: &Element) -> ComputedValues {
        self.resolve_with_parent(element, None)
    }

    /// Resolve computed styles for multiple elements in parallel (uses rayon).
    ///
    /// Each element is resolved independently (no parent-child relationships
    /// are assumed — use [`resolve_with_parent`](Self::resolve_with_parent)
    /// for individual elements when parent style is needed for inheritance).
    #[must_use]
    pub fn resolve_parallel(&self, elements: &[Element]) -> Vec<ComputedValues> {
        use rayon::prelude::*;
        elements
            .par_iter()
            .map(|element| self.resolve(element))
            .collect()
    }

    /// Collect declarations from all rules in a stylesheet that match an element.
    fn collect_matching_declarations(
        &self,
        cascade: &mut Cascade,
        sheet: &CssStylesheet,
        element: &Element,
    ) {
        for rule in &sheet.rules {
            self.collect_from_rule(cascade, rule, element);
        }
    }

    /// Collect declarations from a single rule (or nested rules inside at-rules).
    fn collect_from_rule(
        &self,
        cascade: &mut Cascade,
        rule: &CssRule,
        element: &Element,
    ) {
        match rule {
            CssRule::Style(style_rule) => {
                if rule_matches_element(style_rule, element) {
                    let spec = rule_max_specificity(style_rule);
                    cascade.add_declarations(&style_rule.declarations, spec);
                }
            }
            CssRule::Media(media_rule) => {
                // Always match media rules in the CSS engine (we don't have viewport info here).
                for nested in &media_rule.rules {
                    self.collect_from_rule(cascade, nested, element);
                }
            }
            CssRule::Supports(supports_rule) => {
                for nested in &supports_rule.rules {
                    self.collect_from_rule(cascade, nested, element);
                }
            }
            // Other at-rules (font-face, keyframes, import) do not contribute
            // declarations to the cascade for element styles.
            CssRule::FontFace(_)
            | CssRule::Keyframes(_)
            | CssRule::Import(_)
            | CssRule::Charset(_)
            | CssRule::Namespace(_)
            | CssRule::Page(_)
            | CssRule::Other(_) => {}
        }
    }

    /// Clear all registered stylesheets.
    pub fn clear(&mut self) {
        self.stylesheets.clear();
    }
}

impl Default for StyleResolver {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
use super::*;
use motarjim_ast::css::{CssRule, CssStylesheet, Declaration};
use motarjim_ast::selector::{Selector, SimpleSelector};
use motarjim_ast::Element;
use smol_str::SmolStr;

    fn div_el() -> Element {
        Element::new("div")
    }

    fn span_el() -> Element {
        Element::new("span")
    }

    fn make_style_rule(selector_str: &str, decls: Vec<Declaration>) -> StyleRule {
        let simple = SimpleSelector::Type(SmolStr::new(selector_str));
        StyleRule {
            selectors: vec![Selector {
                simple_selectors: vec![simple],
                combinators: vec![],
            }],
            declarations: decls.into(),
        }
    }

    fn make_css_rule(selector_str: &str, decls: Vec<Declaration>) -> CssRule {
        CssRule::Style(make_style_rule(selector_str, decls))
    }

    fn decl(prop: &str, value: &str) -> Declaration {
        Declaration {
            property: SmolStr::new(prop),
            value: value.to_string(),
            important: false,
        }
    }

    fn important_decl(prop: &str, value: &str) -> Declaration {
        Declaration {
            property: SmolStr::new(prop),
            value: value.to_string(),
            important: true,
        }
    }

    fn sheet(rules: Vec<CssRule>) -> CssStylesheet {
        CssStylesheet {
            rules,
            source_path: None,
        }
    }

    // -----------------------------------------------------------------------
    // Cascade order tests
    // -----------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_cascade_later_overrides_earlier() {
        let mut resolver = StyleResolver::new();
        resolver.add_stylesheet(sheet(vec![
            make_css_rule("div", vec![decl("color", "red")]),
            make_css_rule("div", vec![decl("color", "blue")]),
        ]));
        let cv = resolver.resolve(&div_el());
        assert_eq!(cv.style.color.as_deref(), Some("blue"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_cascade_important_overrides_normal() {
        let mut resolver = StyleResolver::new();
        resolver.add_stylesheet(sheet(vec![
            make_css_rule("div", vec![important_decl("color", "red")]),
            make_css_rule("div", vec![decl("color", "blue")]),
        ]));
        let cv = resolver.resolve(&div_el());
        assert_eq!(cv.style.color.as_deref(), Some("red"));
    }

    // -----------------------------------------------------------------------
    // Specificity tests
    // -----------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_specificity_id_wins_over_class() {
        let mut resolver = StyleResolver::new();
        // ID selector: #main
        let id_rule = StyleRule {
            selectors: vec![Selector {
                simple_selectors: vec![SimpleSelector::Id(SmolStr::new("main"))],
                combinators: vec![],
            }],
            declarations: smallvec::smallvec![decl("color", "green")],
        };
        // Class selector: .highlight
        let class_rule = StyleRule {
            selectors: vec![Selector {
                simple_selectors: vec![SimpleSelector::Class(SmolStr::new("highlight"))],
                combinators: vec![],
            }],
            declarations: smallvec::smallvec![decl("color", "yellow")],
        };
        // Note: style rule with class comes second (later source order),
        // but ID should still win due to higher specificity.
        resolver.add_stylesheet(sheet(vec![
            CssRule::Style(id_rule),
            CssRule::Style(class_rule),
        ]));

        let mut el = div_el();
        el.id = Some(SmolStr::new("main"));
        el.classes.push(SmolStr::new("highlight"));
        let cv = resolver.resolve(&el);
        assert_eq!(cv.style.color.as_deref(), Some("green"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_specificity_multiple_classes_vs_id() {
        let mut resolver = StyleResolver::new();
        // .a.b has specificity (0,2,0)
        let class_rule = StyleRule {
            selectors: vec![Selector {
                simple_selectors: vec![
                    SimpleSelector::Class(SmolStr::new("a")),
                    SimpleSelector::Class(SmolStr::new("b")),
                ],
                combinators: vec![],
            }],
            declarations: smallvec::smallvec![decl("color", "purple")],
        };
        // #id has specificity (1,0,0)
        let id_rule = StyleRule {
            selectors: vec![Selector {
                simple_selectors: vec![SimpleSelector::Id(SmolStr::new("id"))],
                combinators: vec![],
            }],
            declarations: smallvec::smallvec![decl("color", "orange")],
        };
        resolver.add_stylesheet(sheet(vec![CssRule::Style(class_rule), CssRule::Style(id_rule)]));

        let mut el = div_el();
        el.id = Some(SmolStr::new("id"));
        el.classes.push(SmolStr::new("a"));
        el.classes.push(SmolStr::new("b"));
        let cv = resolver.resolve(&el);
        assert_eq!(cv.style.color.as_deref(), Some("orange"));
    }

    // -----------------------------------------------------------------------
    // Computed values tests
    // -----------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_computed_display() {
        let mut resolver = StyleResolver::new();
        resolver.add_stylesheet(sheet(vec![make_css_rule(
            "div",
            vec![decl("display", "flex")],
        )]));
        let cv = resolver.resolve(&div_el());
        assert_eq!(cv.style.display, DisplayType::Flex);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_computed_margin_padding() {
        let mut resolver = StyleResolver::new();
        resolver.add_stylesheet(sheet(vec![make_css_rule(
            "div",
            vec![decl("margin", "10px"), decl("padding", "5px 10px")],
        )]));
        let cv = resolver.resolve(&div_el());
        assert_eq!(cv.style.margin.top, 10.0);
        assert_eq!(cv.style.margin.right, 10.0);
        assert_eq!(cv.style.margin.bottom, 10.0);
        assert_eq!(cv.style.margin.left, 10.0);
        assert_eq!(cv.style.padding.top, 5.0);
        assert_eq!(cv.style.padding.right, 10.0);
        assert_eq!(cv.style.padding.bottom, 5.0);
        assert_eq!(cv.style.padding.left, 10.0);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_computed_flex_properties() {
        let mut resolver = StyleResolver::new();
        resolver.add_stylesheet(sheet(vec![make_css_rule(
            "div",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("justify-content", "center"),
                decl("align-items", "center"),
                decl("flex-grow", "1"),
                decl("flex-wrap", "wrap"),
            ],
        )]));
        let cv = resolver.resolve(&div_el());
        assert_eq!(cv.style.display, DisplayType::Flex);
        assert_eq!(cv.style.flex_direction, Some(FlexDirection::Column));
        assert_eq!(cv.style.justify_content, Some(JustifyContent::Center));
        assert_eq!(cv.style.align_items, Some(AlignItems::Center));
        assert!((cv.style.flex_grow - 1.0).abs() < f64::EPSILON);
        assert_eq!(cv.style.flex_wrap, Some(FlexWrap::Wrap));
    }

    // -----------------------------------------------------------------------
    // Inheritance tests
    // -----------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_inheritance_color() {
        // Color is inherited. If parent has a color, child should inherit it.
        let parent_style = {
            let mut cv = ComputedValues::new();
            cv.style.color = Some("red".to_string());
            cv
        };

        let mut resolver = StyleResolver::new();
        // Child element has no matching rule for color, so it should inherit from parent.
        resolver.add_stylesheet(sheet(vec![make_css_rule(
            "span",
            vec![decl("font-size", "14px")],
        )]));

        let cv = resolver.resolve_with_parent(&span_el(), Some(&parent_style));
        assert_eq!(cv.style.color.as_deref(), Some("red"));
        assert_eq!(cv.style.font_size.as_deref(), Some("14px"));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_child_override_inherited() {
        let parent_style = {
            let mut cv = ComputedValues::new();
            cv.style.color = Some("red".to_string());
            cv.style.font_size = Some("16px".to_string());
            cv
        };

        let mut resolver = StyleResolver::new();
        // Child overrides color but not font-size.
        resolver.add_stylesheet(sheet(vec![make_css_rule(
            "span",
            vec![decl("color", "blue")],
        )]));

        let cv = resolver.resolve_with_parent(&span_el(), Some(&parent_style));
        assert_eq!(cv.style.color.as_deref(), Some("blue"));
        assert_eq!(cv.style.font_size.as_deref(), Some("16px"));
    }

    // -----------------------------------------------------------------------
    // Parallel matching test
    // -----------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parallel_resolve() {
        let mut resolver = StyleResolver::new();
        resolver.add_stylesheet(sheet(vec![
            make_css_rule("div", vec![decl("color", "red"), decl("font-size", "12px")]),
            make_css_rule("span", vec![decl("color", "blue")]),
        ]));

        let elements = vec![div_el(), span_el(), div_el(), span_el()];
        let results = resolver.resolve_parallel(&elements);
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].style.color.as_deref(), Some("red"));
        assert_eq!(results[1].style.color.as_deref(), Some("blue"));
        assert_eq!(results[2].style.color.as_deref(), Some("red"));
        assert_eq!(results[3].style.color.as_deref(), Some("blue"));
        assert_eq!(results[0].style.font_size.as_deref(), Some("12px"));
        assert!(results[1].style.font_size.is_none());
    }

    // -----------------------------------------------------------------------
    // Color parsing tests
    // -----------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_named_color() {
        let c = parse_color("red").expect("should parse red");
        assert_eq!(c, CssColor::Rgba(255, 0, 0, 1.0));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_hex_color() {
        let c = parse_color("#ff0000").expect("should parse hex");
        assert_eq!(c, CssColor::Hex(255, 0, 0, 255));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_hex_short_color() {
        let c = parse_color("#f00").expect("should parse short hex");
        assert_eq!(c, CssColor::Hex(255, 0, 0, 255));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_hex_alpha_color() {
        let c = parse_color("#ff000080").expect("should parse hex with alpha");
        assert_eq!(c, CssColor::Hex(255, 0, 0, 128));
    }

    #[test]
    fn test_parse_rgb_color() {
        let c = parse_color("rgb(255, 0, 0)").expect("should parse rgb");
        assert_eq!(c, CssColor::Rgba(255, 0, 0, 1.0));
    }

    #[test]
    fn test_parse_rgba_color() {
        let c = parse_color("rgba(255, 0, 0, 0.5)").expect("should parse rgba");
        assert_eq!(c, CssColor::Rgba(255, 0, 0, 0.5));
    }

    #[test]
    fn test_parse_transparent() {
        let c = parse_color("transparent").expect("should parse transparent");
        assert_eq!(c, CssColor::Transparent);
    }

    #[test]
    fn test_parse_invalid_color() {
        assert!(parse_color("not-a-color").is_none());
    }

    // -----------------------------------------------------------------------
    // Length / unit parsing tests
    // -----------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_length_px() {
        let l = parse_length("42px").expect("should parse px");
        assert_eq!(l, CssLength::Px(42.0));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_length_em() {
        let l = parse_length("1.5em").expect("should parse em");
        assert_eq!(l, CssLength::Em(1.5));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_length_rem() {
        let l = parse_length("2rem").expect("should parse rem");
        assert_eq!(l, CssLength::Rem(2.0));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse_length_percent() {
        let l = parse_length("50%").expect("should parse percent");
        assert_eq!(l, CssLength::Percent(50.0));
    }

    #[test]
    fn test_parse_length_auto_returns_none() {
        assert!(parse_length("auto").is_none());
    }

    #[test]
    fn test_parse_length_invalid() {
        assert!(parse_length("abc").is_none());
    }

    // -----------------------------------------------------------------------
    // ComputedValues builder test
    // -----------------------------------------------------------------------

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_from_map_basic() {
        let mut map = HashMap::new();
        map.insert(SmolStr::new("color"), "green".to_string());
        map.insert(SmolStr::new("display"), "flex".to_string());
        map.insert(SmolStr::new("opacity"), "0.75".to_string());

        let cv = ComputedValues::from_map(&map, None);
        assert_eq!(cv.style.color.as_deref(), Some("green"));
        assert_eq!(cv.style.display, DisplayType::Flex);
        assert!((cv.style.opacity - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_resolver_clear() {
        let mut resolver = StyleResolver::new();
        resolver.add_stylesheet(sheet(vec![make_css_rule(
            "div",
            vec![decl("color", "red")],
        )]));
        resolver.clear();
        assert!(resolver.stylesheets().is_empty());
        let cv = resolver.resolve(&div_el());
        assert!(cv.style.color.is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_font_weight_parsing() {
        let fw = parse_font_weight("bold");
        assert_eq!(fw, Some(FontWeight::Bold));

        let fw = parse_font_weight("700");
        assert_eq!(fw, Some(FontWeight::Bold));

        let fw = parse_font_weight("400");
        assert_eq!(fw, Some(FontWeight::Normal));

        let fw = parse_font_weight("850");
        assert_eq!(fw, Some(FontWeight::Custom(850)));
    }
}
