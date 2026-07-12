use crate::*;

/// Applies every declaration in `map` to `style`, one property at a time.
///
/// Custom properties (`--*`) are stored in `style.custom_properties` first,
/// then all values are resolved for `var()` references.
pub(crate) fn apply_declarations(style: &mut ComputedStyle, map: &HashMap<SmolStr, String>) {
    // Phase 1: Collect all custom properties (variables) first.
    for (prop, value) in map {
        if prop.starts_with("--") {
            let name = prop[2..].to_string();
            style.custom_properties.insert(name, value.clone());
        }
    }

    // Phase 2: Apply standard properties with var() resolution.
    for (prop, value) in map {
        if !prop.starts_with("--") {
            let resolved = resolve_var(value, style);
            apply_property(style, prop.as_str(), &resolved);
        }
    }
}

/// Resolve `var()` references in a CSS value string.
///
/// Supports:
/// - `var(--name)` — substitute with custom property value
/// - `var(--name, fallback)` — use fallback if variable is undefined
/// - Nested `var()` — resolve recursively with cycle detection
#[must_use]
pub fn resolve_var(value: &str, style: &ComputedStyle) -> String {
    let mut visited = std::collections::HashSet::new();
    resolve_var_inner(value, style, &mut visited)
}

fn resolve_var_inner(value: &str, style: &ComputedStyle, visited: &mut std::collections::HashSet<String>) -> String {
    let mut result = value.to_string();

    for _ in 0..20 {
        let var_start = match result.find("var(") {
            Some(pos) => pos,
            None => break,
        };

        let open_paren = var_start + 4;
        let mut depth_count = 1;
        let mut var_end = open_paren;
        for (i, ch) in result[open_paren..].char_indices() {
            match ch {
                '(' => depth_count += 1,
                ')' => {
                    depth_count -= 1;
                    if depth_count == 0 {
                        var_end = open_paren + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth_count != 0 {
            break;
        }

        let inner = &result[open_paren..var_end];
        let (var_name, fallback) = match split_var_args(inner) {
            Some((name, fb)) => (name.to_string(), Some(fb.to_string())),
            None => (inner.trim().to_string(), None),
        };

        let lookup_name = var_name.strip_prefix("--").unwrap_or(&var_name).to_string();

        // Cycle detection: if we've already tried resolving this variable, stop
        if !visited.insert(lookup_name.clone()) {
            break;
        }

        let replacement = if let Some(val) = style.custom_properties.get(&lookup_name) {
            resolve_var_inner(val, style, visited)
        } else if let Some(fb) = fallback {
            fb
        } else {
            format!("var({inner})")
        };

        let original_var = &result[var_start..var_end + 1];
        if replacement == original_var {
            break;
        }

        let before = &result[..var_start];
        let after = &result[var_end + 1..];
        result = format!("{before}{replacement}{after}");
    }

    result
}

/// Split the inner content of `var(name, fallback)` into (name, fallback).
fn split_var_args(inner: &str) -> Option<(&str, &str)> {
    let inner = inner.trim();
    // Find comma at depth 0 (not inside nested parens)
    let mut paren_depth = 0;
    for (i, ch) in inner.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            ',' if paren_depth == 0 => {
                let name = inner[..i].trim();
                let fallback = inner[i + 1..].trim();
                return Some((name, fallback));
            }
            _ => {}
        }
    }
    None
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
        "grid-template-columns" => {
            style.grid_template_columns = Some(v.to_string());
            if let Some(template) = parse_grid_template(v) {
                style.grid_template_columns_structured = Some(template);
            }
        }
        "grid-template-rows" => {
            style.grid_template_rows = Some(v.to_string());
            if let Some(template) = parse_grid_template(v) {
                style.grid_template_rows_structured = Some(template);
            }
        }
        "grid-column" => {
            style.grid_column = Some(v.to_string());
            if let Some((start, end)) = parse_grid_placement(v) {
                style.grid_column_start = Some(start);
                if let Some(e) = end {
                    style.grid_column_end = Some(e);
                }
            }
        }
        "grid-row" => {
            style.grid_row = Some(v.to_string());
            if let Some((start, end)) = parse_grid_placement(v) {
                style.grid_row_start = Some(start);
                if let Some(e) = end {
                    style.grid_row_end = Some(e);
                }
            }
        }
        "grid-template-areas" => {
            style.grid_template_areas = Some(parse_grid_template_areas(v));
        }
        "grid-auto-flow" => style.grid_auto_flow = Some(v.to_string()),
        "grid-auto-columns" => {
            if let Some(template) = parse_grid_template(v) {
                style.grid_auto_columns = Some(template);
            }
        }
        "grid-auto-rows" => {
            if let Some(template) = parse_grid_template(v) {
                style.grid_auto_rows = Some(template);
            }
        }
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
        // Positioning offsets (TASK 2)
        "top" => style.top = Some(v.to_string()),
        "right" => style.right = Some(v.to_string()),
        "bottom" => style.bottom = Some(v.to_string()),
        "left" => style.left = Some(v.to_string()),
        "inset" => {
            let parts: Vec<&str> = v.split_whitespace().collect();
            match parts.len() {
                1 => {
                    style.top = Some(parts[0].to_string());
                    style.right = Some(parts[0].to_string());
                    style.bottom = Some(parts[0].to_string());
                    style.left = Some(parts[0].to_string());
                }
                2 => {
                    style.top = Some(parts[0].to_string());
                    style.right = Some(parts[1].to_string());
                    style.bottom = Some(parts[0].to_string());
                    style.left = Some(parts[1].to_string());
                }
                3 => {
                    style.top = Some(parts[0].to_string());
                    style.right = Some(parts[1].to_string());
                    style.bottom = Some(parts[2].to_string());
                    style.left = Some(parts[1].to_string());
                }
                4 => {
                    style.top = Some(parts[0].to_string());
                    style.right = Some(parts[1].to_string());
                    style.bottom = Some(parts[2].to_string());
                    style.left = Some(parts[3].to_string());
                }
                _ => {}
            }
        }
        // Animation properties (TASK 6)
        "animation-name" => style.animation_name = Some(v.to_string()),
        "animation-duration" => style.animation_duration = Some(v.to_string()),
        "animation-timing-function" => style.animation_timing_function = Some(v.to_string()),
        "animation-delay" => style.animation_delay = Some(v.to_string()),
        "animation-iteration-count" => style.animation_iteration_count = Some(v.to_string()),
        "animation-direction" => style.animation_direction = Some(v.to_string()),
        "animation-fill-mode" => style.animation_fill_mode = Some(v.to_string()),
        "animation-play-state" => style.animation_play_state = Some(v.to_string()),
        "animation" => {
            // Shorthand: name duration timing-function delay iteration-count direction fill-mode play-state
            let parts: Vec<&str> = v.split_whitespace().collect();
            let mut name = None;
            let mut duration = None;
            let mut timing = None;
            let mut delay = None;
            let mut iteration = None;
            let mut direction = None;
            let mut fill_mode = None;
            let mut play_state = None;

            for part in &parts {
                if part.starts_with("normal") || part.starts_with("reverse")
                    || part.starts_with("alternate") || part.starts_with("alternate-reverse")
                {
                    direction = Some(part.to_string());
                } else if part.starts_with("forwards") || part.starts_with("backwards")
                    || part.starts_with("both")
                {
                    fill_mode = Some(part.to_string());
                } else if part.starts_with("paused") || part.starts_with("running") {
                    play_state = Some(part.to_string());
                } else if part.starts_with("infinite")
                    || part.chars().all(|c| c.is_ascii_digit())
                {
                    iteration = Some(part.to_string());
                } else if part.contains("ease") || part.contains("linear")
                    || part.contains("step") || *part == "cubic-bezier"
                {
                    timing = Some(part.to_string());
                } else if part.ends_with("ms") || part.ends_with("s") || *part == "0s" || *part == "0ms" {
                    if delay.is_none() && duration.is_some() {
                        delay = Some(part.to_string());
                    } else {
                        duration = Some(part.to_string());
                    }
                } else {
                    // Assume it's the animation name
                    name = Some(part.to_string());
                }
            }

            style.animation_name = name;
            style.animation_duration = duration;
            style.animation_timing_function = timing;
            style.animation_delay = delay;
            style.animation_iteration_count = iteration;
            style.animation_direction = direction;
            style.animation_fill_mode = fill_mode;
            style.animation_play_state = play_state;
        }
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
// Grid value parsers
// ---------------------------------------------------------------------------

/// Parse a `grid-template-columns` / `grid-template-rows` value into a structured `GridTemplate`.
///
/// Supports: `1fr`, `200px`, `auto`, `min-content`, `max-content`, `repeat()`, `minmax()`.
#[must_use]
pub fn parse_grid_template(value: &str) -> Option<motarjim_ast_html::grid::GridTemplate> {
    use motarjim_ast_html::grid::{GridTemplate, GridTrack};

    let tokens = tokenize_grid_value(value);
    let tracks: Vec<GridTrack> = tokens
        .iter()
        .filter_map(|token| parse_grid_track(token))
        .collect();

    if tracks.is_empty() {
        None
    } else {
        Some(GridTemplate { tracks })
    }
}

/// Tokenize a CSS grid template value, respecting parenthesized groups.
fn tokenize_grid_value(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in value.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ' ' if depth == 0 => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn parse_grid_track(token: &str) -> Option<motarjim_ast_html::grid::GridTrack> {
    use motarjim_ast_html::grid::GridTrack;

    let token = token.trim().to_lowercase();

    if token == "auto" {
        return Some(GridTrack::Auto);
    }
    if token == "min-content" {
        return Some(GridTrack::MinContent);
    }
    if token == "max-content" {
        return Some(GridTrack::MaxContent);
    }

    // `1fr`, `2.5fr`, etc.
    if let Some(fr_str) = token.strip_suffix("fr") {
        if let Ok(val) = fr_str.trim().parse::<f64>() {
            return Some(GridTrack::Fr(val));
        }
    }

    // `200px`, `10em`, etc.
    if let Some(px_str) = token.strip_suffix("px") {
        if let Ok(val) = px_str.trim().parse::<f64>() {
            return Some(GridTrack::Fixed(val));
        }
    }

    // `fit-content(200px)`
    if token.starts_with("fit-content(") && token.ends_with(')') {
        let inner = &token[12..token.len() - 1];
        if let Some(val) = parse_track_length(inner) {
            return Some(GridTrack::FitContent(val));
        }
    }

    // `minmax(min, max)`
    if token.starts_with("minmax(") && token.ends_with(')') {
        let inner = &token[7..token.len() - 1];
        if let Some(comma_pos) = find_comma_at_depth_zero(inner) {
            let min_str = inner[..comma_pos].trim();
            let max_str = inner[comma_pos + 1..].trim();
            let min = parse_grid_track_token(min_str);
            let max = parse_grid_track_token(max_str);
            if let (Some(min_track), Some(max_track)) = (min, max) {
                return Some(GridTrack::MinMax(Box::new(min_track), Box::new(max_track)));
            }
        }
    }

    // `repeat(count, tracks)`
    if token.starts_with("repeat(") && token.ends_with(')') {
        let inner = &token[7..token.len() - 1];
        if let Some(comma_pos) = find_comma_at_depth_zero(inner) {
            let count_str = inner[..comma_pos].trim();
            let tracks_str = inner[comma_pos + 1..].trim();
            if let Ok(count) = count_str.parse::<u32>() {
                let tracks: Vec<GridTrack> = tracks_str
                    .split_whitespace()
                    .filter_map(|t| parse_grid_track_token(t))
                    .collect();
                if !tracks.is_empty() {
                    return Some(GridTrack::Repeat(count, tracks));
                }
            }
        }
    }

    // Plain number as pixel value
    if let Ok(val) = token.parse::<f64>() {
        return Some(GridTrack::Fixed(val));
    }

    None
}

fn parse_grid_track_token(token: &str) -> Option<motarjim_ast_html::grid::GridTrack> {
    parse_grid_track(token)
}

fn parse_track_length(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(px_str) = s.strip_suffix("px") {
        px_str.trim().parse::<f64>().ok()
    } else {
        s.parse::<f64>().ok()
    }
}

/// Parse a `grid-column` or `grid-row` value.
///
/// Supports: `1 / 3`, `span 2`, `auto`, `1 / span 2`.
#[must_use]
pub fn parse_grid_placement(
    value: &str,
) -> Option<(
    motarjim_ast_html::grid::GridPlacement,
    Option<motarjim_ast_html::grid::GridPlacement>,
)> {
    use motarjim_ast_html::grid::{GridLine, GridPlacement};

    let value = value.trim();

    if value == "auto" {
        return Some((
            GridPlacement {
                line: GridLine::Auto,
                span: None,
            },
            None,
        ));
    }

    // `span N`
    if let Some(span_str) = value.strip_prefix("span ") {
        if let Ok(span) = span_str.trim().parse::<u32>() {
            return Some((
                GridPlacement {
                    line: GridLine::Auto,
                    span: Some(span),
                },
                None,
            ));
        }
    }

    // `1 / 3` or `1 / span 2`
    if let Some(slash_pos) = value.find('/') {
        let start_str = value[..slash_pos].trim();
        let end_str = value[slash_pos + 1..].trim();

        let start = parse_grid_line(start_str);
        let end = parse_grid_line(end_str);

        return Some((start, Some(end)));
    }

    // Just a number
    Some((
        parse_grid_line(value),
        None,
    ))
}

fn parse_grid_line(s: &str) -> motarjim_ast_html::grid::GridPlacement {
    use motarjim_ast_html::grid::{GridLine, GridPlacement};

    let s = s.trim();
    if s == "auto" {
        GridPlacement {
            line: GridLine::Auto,
            span: None,
        }
    } else if let Some(span_str) = s.strip_prefix("span ") {
        if let Ok(span) = span_str.trim().parse::<u32>() {
            GridPlacement {
                line: GridLine::Auto,
                span: Some(span),
            }
        } else {
            GridPlacement {
                line: GridLine::Auto,
                span: None,
            }
        }
    } else if let Ok(num) = s.parse::<u32>() {
        GridPlacement {
            line: GridLine::Number(num),
            span: None,
        }
    } else {
        GridPlacement {
            line: GridLine::Named(s.to_string()),
            span: None,
        }
    }
}

/// Parse `grid-template-areas` value like `"a b" "c d"`.
#[must_use]
pub fn parse_grid_template_areas(value: &str) -> Vec<String> {
    let mut areas = Vec::new();
    let mut in_quotes = false;
    let mut current = String::new();

    for ch in value.chars() {
        match ch {
            '"' if !in_quotes => {
                in_quotes = true;
                current.clear();
            }
            '"' if in_quotes => {
                in_quotes = false;
                for area in current.split_whitespace() {
                    areas.push(area.to_string());
                }
            }
            _ if in_quotes => {
                current.push(ch);
            }
            _ => {}
        }
    }

    areas
}

/// Find the position of a comma at depth 0 (not inside parentheses).
fn find_comma_at_depth_zero(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Selector matching helpers
// ---------------------------------------------------------------------------
