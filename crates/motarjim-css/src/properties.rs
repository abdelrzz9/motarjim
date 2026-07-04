use crate::*;

pub(crate) fn apply_declarations(style: &mut ComputedStyle, map: &HashMap<SmolStr, String>) {
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
            if v != "auto" =>
        {
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
        "flex-basis" if v != "auto" => {
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
        "gap" | "row-gap" | "column-gap" if v != "normal" => match property {
            "gap" => style.gap = Some(v.to_string()),
            "row-gap" => style.row_gap = Some(v.to_string()),
            "column-gap" => style.column_gap = Some(v.to_string()),
            _ => {}
        },
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
pub(crate) fn parse_font_weight(v: &str) -> Option<FontWeight> {
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
