//! Computed style types for the Motarjim compiler.
#![allow(missing_docs)]

/// CSS display types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum DisplayType {
    Block,
    Inline,
    InlineBlock,
    Flex,
    Grid,
    None,
    Contents,
    Flow,
    FlowRoot,
    Table,
    TableRow,
    TableCell,
    ListItem,
}

/// CSS position types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum PositionType {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

/// CSS flex direction values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// CSS flex wrap values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// CSS justify-content values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// CSS align-items values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

/// CSS align-content values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
}

/// CSS text-align values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

/// CSS font-weight values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Custom(u16),
}

/// CSS overflow values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// A box with four edge values (top, right, bottom, left).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct EdgeValues {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl EdgeValues {
    /// Creates edge values with the same value on all sides.
    #[must_use]
    pub const fn all(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Creates edge values with individual values per side.
    #[must_use]
    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Creates edge values from vertical/horizontal pairs.
    #[must_use]
    pub const fn symmetric(vertical: f64, horizontal: f64) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl Default for EdgeValues {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

/// CSS background properties.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Background {
    pub color: Option<String>,
    pub image: Option<String>,
    pub position: Option<String>,
    pub repeat: Option<String>,
    pub size: Option<String>,
}

/// CSS border properties.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Border {
    pub width: EdgeValues,
    pub color: Option<String>,
    pub style: Option<String>,
    pub radius: EdgeValues,
}

/// The resolved computed style for a node after CSS cascading and inheritance.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct ComputedStyle {
    pub display: DisplayType,
    pub position: PositionType,
    pub width: Option<String>,
    pub height: Option<String>,
    pub min_width: Option<String>,
    pub min_height: Option<String>,
    pub max_width: Option<String>,
    pub max_height: Option<String>,
    pub margin: EdgeValues,
    pub padding: EdgeValues,
    pub flex_direction: Option<FlexDirection>,
    pub flex_wrap: Option<FlexWrap>,
    pub flex_grow: f64,
    pub flex_shrink: f64,
    pub flex_basis: Option<String>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub align_content: Option<AlignContent>,
    pub align_self: Option<AlignItems>,
    pub gap: Option<String>,
    pub row_gap: Option<String>,
    pub column_gap: Option<String>,
    pub grid_template_columns: Option<String>,
    pub grid_template_rows: Option<String>,
    pub grid_column: Option<String>,
    pub grid_row: Option<String>,
    pub color: Option<String>,
    pub background: Option<Background>,
    pub border: Option<Border>,
    pub font_family: Option<String>,
    pub font_size: Option<String>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<String>,
    pub line_height: Option<String>,
    pub text_align: Option<TextAlign>,
    pub text_decoration: Option<String>,
    pub opacity: f64,
    pub overflow: Option<Overflow>,
    pub cursor: Option<String>,
    pub box_shadow: Option<String>,
    pub transform: Option<String>,
    pub transition: Option<String>,
    pub visibility: bool,
    pub z_index: Option<i32>,
    pub pointer_events: Option<String>,
    pub resize: Option<String>,
    pub user_select: Option<String>,
    pub appearance: Option<String>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: DisplayType::Block,
            position: PositionType::Static,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            margin: EdgeValues::default(),
            padding: EdgeValues::default(),
            flex_direction: None,
            flex_wrap: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
            justify_content: None,
            align_items: None,
            align_content: None,
            align_self: None,
            gap: None,
            row_gap: None,
            column_gap: None,
            grid_template_columns: None,
            grid_template_rows: None,
            grid_column: None,
            grid_row: None,
            color: None,
            background: None,
            border: None,
            font_family: None,
            font_size: None,
            font_weight: None,
            font_style: None,
            line_height: None,
            text_align: None,
            text_decoration: None,
            opacity: 1.0,
            overflow: None,
            cursor: None,
            box_shadow: None,
            transform: None,
            transition: None,
            visibility: true,
            z_index: None,
            pointer_events: None,
            resize: None,
            user_select: None,
            appearance: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_values() {
        let ev = EdgeValues::default();
        assert_eq!(ev.top, 0.0);
        assert_eq!(EdgeValues::all(10.0).top, 10.0);
        assert_eq!(EdgeValues::symmetric(5.0, 10.0).right, 10.0);
        assert_eq!(EdgeValues::new(1.0, 2.0, 3.0, 4.0).bottom, 3.0);
    }

    #[test]
    fn test_computed_style_defaults() {
        let s = ComputedStyle::default();
        assert_eq!(s.display, DisplayType::Block);
        assert_eq!(s.position, PositionType::Static);
        assert_eq!(s.flex_grow, 0.0);
        assert_eq!(s.flex_shrink, 1.0);
        assert_eq!(s.opacity, 1.0);
        assert!(s.visibility);
        assert!(s.width.is_none());
        assert!(s.color.is_none());
        assert!(s.background.is_none());
        assert!(s.border.is_none());
    }

    #[test]
    fn test_computed_style_with_values() {
        let mut s = ComputedStyle::default();
        s.display = DisplayType::Flex;
        s.flex_direction = Some(FlexDirection::Row);
        s.justify_content = Some(JustifyContent::Center);
        s.align_items = Some(AlignItems::Center);
        s.color = Some("#333".to_string());
        s.opacity = 0.8;
        assert_eq!(s.display, DisplayType::Flex);
        assert_eq!(s.flex_direction, Some(FlexDirection::Row));
        assert_eq!(s.justify_content, Some(JustifyContent::Center));
        assert_eq!(s.color.as_deref(), Some("#333"));
    }

    #[test]
    fn test_enum_variants() {
        assert_eq!(
            [
                DisplayType::Block,
                DisplayType::Inline,
                DisplayType::Flex,
                DisplayType::Grid,
                DisplayType::None
            ]
            .len(),
            5
        );
        assert!(matches!(FontWeight::Custom(450), FontWeight::Custom(v) if v == 450));
        assert!(matches!(FontWeight::Normal, FontWeight::Normal));
    }
}
