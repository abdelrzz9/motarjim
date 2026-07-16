use motarjim_ast_css::{MediaCondition, MediaQuery};

/// Evaluate whether a media query matches the given viewport dimensions and
/// color scheme preference.
///
/// Returns `true` if the query should be applied.
#[must_use]
pub fn evaluate_media_query(query: &MediaQuery, viewport: (u32, u32), color_scheme: &str) -> bool {
    if query.conditions.is_empty() {
        return true;
    }

    // All conditions must be satisfied (implicit AND between top-level conditions)
    query
        .conditions
        .iter()
        .all(|cond| evaluate_condition(cond, viewport, color_scheme))
}

fn evaluate_condition(condition: &MediaCondition, viewport: (u32, u32), color_scheme: &str) -> bool {
    match condition {
        MediaCondition::All => true,
        MediaCondition::Screen => true,
        MediaCondition::Print => false,
        MediaCondition::Speech => false,
        MediaCondition::Not(inner) => !evaluate_condition(inner, viewport, color_scheme),
        MediaCondition::Only(inner) => evaluate_condition(inner, viewport, color_scheme),
        MediaCondition::And(conds) => conds
            .iter()
            .all(|c| evaluate_condition(c, viewport, color_scheme)),
        MediaCondition::Or(conds) => conds
            .iter()
            .any(|c| evaluate_condition(c, viewport, color_scheme)),
        MediaCondition::MinWidth(px) => parse_px_value(px) <= Some(viewport.0 as f64),
        MediaCondition::MaxWidth(px) => parse_px_value(px) >= Some(viewport.0 as f64),
        MediaCondition::MinHeight(px) => parse_px_value(px) <= Some(viewport.1 as f64),
        MediaCondition::MaxHeight(px) => parse_px_value(px) >= Some(viewport.1 as f64),
        MediaCondition::Orientation(val) => {
            let is_portrait = viewport.1 > viewport.0;
            match val.as_str() {
                "portrait" => is_portrait,
                "landscape" => !is_portrait,
                _ => true,
            }
        }
        MediaCondition::PrefersColorScheme(scheme) => {
            scheme.eq_ignore_ascii_case(color_scheme)
        }
        MediaCondition::Feature { name, value: Some(val) } => {
            // Handle common media features
            match name.as_str() {
                "prefers-color-scheme" => val.eq_ignore_ascii_case(color_scheme),
                "min-width" => parse_px_value(val) <= Some(viewport.0 as f64),
                "max-width" => parse_px_value(val) >= Some(viewport.0 as f64),
                "min-height" => parse_px_value(val) <= Some(viewport.1 as f64),
                "max-height" => parse_px_value(val) >= Some(viewport.1 as f64),
                "orientation" => {
                    let is_portrait = viewport.1 > viewport.0;
                    match val.as_str() {
                        "portrait" => is_portrait,
                        "landscape" => !is_portrait,
                        _ => true,
                    }
                }
                _ => true, // Unknown feature — assume matches
            }
        }
        MediaCondition::Feature { name, value: None } => {
            // Feature without value (e.g., `@media (color)`) — assume matches
            match name.as_str() {
                "print" => false,
                "screen" => true,
                _ => true,
            }
        }
    }
}

/// Parse a pixel value from a CSS string like `"768px"` or `"768"`.
pub fn parse_px_value(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(num) = s.strip_suffix("px") {
        num.trim().parse::<f64>().ok()
    } else {
        s.parse::<f64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_ast_css::{MediaCondition, MediaQuery};

    fn mq(conditions: Vec<MediaCondition>) -> MediaQuery {
        MediaQuery { conditions }
    }

    #[test]
    fn test_empty_query_matches() {
        let q = mq(vec![]);
        assert!(evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_all_matches() {
        let q = mq(vec![MediaCondition::All]);
        assert!(evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_screen_matches() {
        let q = mq(vec![MediaCondition::Screen]);
        assert!(evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_print_does_not_match() {
        let q = mq(vec![MediaCondition::Print]);
        assert!(!evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_max_width_match() {
        let q = mq(vec![MediaCondition::MaxWidth("768px".to_string())]);
        assert!(evaluate_media_query(&q, (375, 667), "light"));
    }

    #[test]
    fn test_max_width_no_match() {
        let q = mq(vec![MediaCondition::MaxWidth("768px".to_string())]);
        assert!(!evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_min_width_match() {
        let q = mq(vec![MediaCondition::MinWidth("1024px".to_string())]);
        assert!(evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_min_width_no_match() {
        let q = mq(vec![MediaCondition::MinWidth("1024px".to_string())]);
        assert!(!evaluate_media_query(&q, (375, 667), "light"));
    }

    #[test]
    fn test_not_negates() {
        let q = mq(vec![MediaCondition::Not(Box::new(
            MediaCondition::MaxWidth("768px".to_string()),
        ))]);
        // Not(max-width: 768px) = not(true) = false for 375px viewport
        assert!(!evaluate_media_query(&q, (375, 667), "light"));
    }

    #[test]
    fn test_and_all_true() {
        let q = mq(vec![MediaCondition::And(vec![
            MediaCondition::Screen,
            MediaCondition::MinWidth("1024px".to_string()),
        ])]);
        assert!(evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_and_one_false() {
        let q = mq(vec![MediaCondition::And(vec![
            MediaCondition::Screen,
            MediaCondition::MaxWidth("768px".to_string()),
        ])]);
        assert!(!evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_or_one_true() {
        let q = mq(vec![MediaCondition::Or(vec![
            MediaCondition::MaxWidth("768px".to_string()),
            MediaCondition::MinWidth("1024px".to_string()),
        ])]);
        assert!(evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_prefers_color_scheme_dark_match() {
        let q = mq(vec![MediaCondition::PrefersColorScheme("dark".to_string())]);
        assert!(evaluate_media_query(&q, (1920, 1080), "dark"));
    }

    #[test]
    fn test_prefers_color_scheme_dark_no_match() {
        let q = mq(vec![MediaCondition::PrefersColorScheme("dark".to_string())]);
        assert!(!evaluate_media_query(&q, (1920, 1080), "light"));
    }

    #[test]
    fn test_parse_px() {
        assert_eq!(parse_px_value("768px"), Some(768.0));
        assert_eq!(parse_px_value("768"), Some(768.0));
        assert_eq!(parse_px_value("10.5px"), Some(10.5));
        assert_eq!(parse_px_value("abc"), None);
    }
}
