//! CSS `calc()` expression evaluator.

/// The result of evaluating a `calc()` expression.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcValue {
    /// A resolved length with a concrete pixel value.
    Length(f64),
    /// A percentage value.
    Percentage(f64),
    /// A unitless number.
    Number(f64),
    /// Cannot be resolved — raw expression kept as-is.
    Raw(String),
}

/// Context for evaluating `calc()` expressions.
#[derive(Debug, Clone, Copy)]
pub struct CalcContext {
    /// Viewport width in pixels.
    pub viewport_width: f64,
    /// Viewport height in pixels.
    pub viewport_height: f64,
    /// Parent element width in pixels (if known).
    pub parent_width: Option<f64>,
    /// Current font size in pixels.
    pub font_size: f64,
}

impl Default for CalcContext {
    fn default() -> Self {
        Self {
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            parent_width: None,
            font_size: 16.0,
        }
    }
}

/// Evaluate a `calc()` expression string.
///
/// Expects the input to be the inner expression (without `calc(` and `)`),
/// e.g., `"100% - 40px"` or `"10px + 20px"`.
#[must_use]
pub fn evaluate_calc(expr: &str, ctx: &CalcContext) -> CalcValue {
    let tokens = tokenize_calc(expr);
    match eval_expression(&tokens, 0, ctx) {
        Some((result, _)) => result,
        None => CalcValue::Raw(format!("calc({expr})")),
    }
}

/// A token in a calc expression.
#[derive(Debug, Clone, PartialEq)]
enum CalcToken {
    Length(f64, LengthUnit),
    Percentage(f64),
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    LParen,
    RParen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LengthUnit {
    Px,
    Em,
    Rem,
    Vw,
    Vh,
    Percent,
    None,
}

fn tokenize_calc(expr: &str) -> Vec<CalcToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' => i += 1,
            '+' => {
                tokens.push(CalcToken::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(CalcToken::Minus);
                i += 1;
            }
            '*' => {
                tokens.push(CalcToken::Multiply);
                i += 1;
            }
            '/' => {
                tokens.push(CalcToken::Divide);
                i += 1;
            }
            '(' => {
                tokens.push(CalcToken::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(CalcToken::RParen);
                i += 1;
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                // Check for unit
                let unit = if i < chars.len() {
                    match chars[i] {
                        '%' => {
                            i += 1;
                            LengthUnit::Percent
                        }
                        'p' if i + 1 < chars.len() && chars[i + 1] == 'x' => {
                            i += 2;
                            LengthUnit::Px
                        }
                        'e' if i + 1 < chars.len() && chars[i + 1] == 'm' => {
                            i += 2;
                            LengthUnit::Em
                        }
                        'r'
                            if i + 2 < chars.len()
                                && chars[i + 1] == 'e'
                                && chars[i + 2] == 'm' =>
                        {
                            i += 3;
                            LengthUnit::Rem
                        }
                        'v' if i + 1 < chars.len() && chars[i + 1] == 'w' => {
                            i += 2;
                            LengthUnit::Vw
                        }
                        'v' if i + 1 < chars.len() && chars[i + 1] == 'h' => {
                            i += 2;
                            LengthUnit::Vh
                        }
                        _ => LengthUnit::None,
                    }
                } else {
                    LengthUnit::None
                };

                if let Ok(num) = chars[start..i - match unit {
                    LengthUnit::Px => 2,
                    LengthUnit::Em => 2,
                    LengthUnit::Rem => 3,
                    LengthUnit::Vw => 2,
                    LengthUnit::Vh => 2,
                    LengthUnit::Percent => 1,
                    LengthUnit::None => 0,
                }]
                    .iter()
                    .collect::<String>()
                    .as_str()
                    .parse::<f64>()
                {
                    match unit {
                        LengthUnit::Percent => tokens.push(CalcToken::Percentage(num)),
                        LengthUnit::None => tokens.push(CalcToken::Number(num)),
                        _ => tokens.push(CalcToken::Length(num, unit)),
                    }
                }
            }
            _ => i += 1, // skip unknown chars
        }
    }

    tokens
}

fn eval_expression(tokens: &[CalcToken], pos: usize, ctx: &CalcContext) -> Option<(CalcValue, usize)> {
    eval_add_sub(tokens, pos, ctx)
}

fn eval_add_sub(tokens: &[CalcToken], pos: usize, ctx: &CalcContext) -> Option<(CalcValue, usize)> {
    let (mut left, mut next_pos) = eval_mul_div(tokens, pos, ctx)?;

    while next_pos < tokens.len() {
        match tokens[next_pos] {
            CalcToken::Plus => {
                let (right, new_pos) = eval_mul_div(tokens, next_pos + 1, ctx)?;
                left = add_values(left, right)?;
                next_pos = new_pos;
            }
            CalcToken::Minus => {
                let (right, new_pos) = eval_mul_div(tokens, next_pos + 1, ctx)?;
                left = sub_values(left, right)?;
                next_pos = new_pos;
            }
            _ => break,
        }
    }

    Some((left, next_pos))
}

fn eval_mul_div(tokens: &[CalcToken], pos: usize, ctx: &CalcContext) -> Option<(CalcValue, usize)> {
    let (mut left, mut next_pos) = eval_primary(tokens, pos, ctx)?;

    while next_pos < tokens.len() {
        match tokens[next_pos] {
            CalcToken::Multiply => {
                let (right, new_pos) = eval_primary(tokens, next_pos + 1, ctx)?;
                left = mul_values(left, right)?;
                next_pos = new_pos;
            }
            CalcToken::Divide => {
                let (right, new_pos) = eval_primary(tokens, next_pos + 1, ctx)?;
                left = div_values(left, right)?;
                next_pos = new_pos;
            }
            _ => break,
        }
    }

    Some((left, next_pos))
}

fn eval_primary(tokens: &[CalcToken], pos: usize, ctx: &CalcContext) -> Option<(CalcValue, usize)> {
    if pos >= tokens.len() {
        return None;
    }

    match &tokens[pos] {
        CalcToken::LParen => {
            let (val, next_pos) = eval_expression(tokens, pos + 1, ctx)?;
            if next_pos < tokens.len() && tokens[next_pos] == CalcToken::RParen {
                Some((val, next_pos + 1))
            } else {
                Some((val, next_pos))
            }
        }
        CalcToken::Length(val, unit) => {
            let px = convert_to_px(*val, *unit, ctx);
            Some((CalcValue::Length(px), pos + 1))
        }
        CalcToken::Percentage(val) => {
            // If we know the parent width, convert percentage to pixels
            if let Some(parent) = ctx.parent_width {
                Some((CalcValue::Length(*val * parent / 100.0), pos + 1))
            } else {
                Some((CalcValue::Percentage(*val), pos + 1))
            }
        }
        CalcToken::Number(val) => Some((CalcValue::Number(*val), pos + 1)),
        CalcToken::Minus => {
            let (val, next_pos) = eval_primary(tokens, pos + 1, ctx)?;
            Some((negate_value(val), next_pos))
        }
        _ => None,
    }
}

/// Convert a CSS length to pixels.
fn convert_to_px(val: f64, unit: LengthUnit, ctx: &CalcContext) -> f64 {
    match unit {
        LengthUnit::Px => val,
        LengthUnit::Em => val * ctx.font_size,
        LengthUnit::Rem => val * 16.0,
        LengthUnit::Vw => val * ctx.viewport_width / 100.0,
        LengthUnit::Vh => val * ctx.viewport_height / 100.0,
        LengthUnit::Percent => {
            if let Some(parent) = ctx.parent_width {
                val * parent / 100.0
            } else {
                val // Can't resolve without parent — keep as raw
            }
        }
        LengthUnit::None => val,
    }
}

fn add_values(a: CalcValue, b: CalcValue) -> Option<CalcValue> {
    match (a, b) {
        (CalcValue::Length(a), CalcValue::Length(b)) => Some(CalcValue::Length(a + b)),
        (CalcValue::Percentage(a), CalcValue::Percentage(b)) => Some(CalcValue::Percentage(a + b)),
        (CalcValue::Number(a), CalcValue::Number(b)) => Some(CalcValue::Number(a + b)),
        (CalcValue::Length(a), CalcValue::Number(b))
        | (CalcValue::Number(b), CalcValue::Length(a)) => Some(CalcValue::Length(a + b)),
        (CalcValue::Percentage(a), CalcValue::Number(b))
        | (CalcValue::Number(b), CalcValue::Percentage(a)) => Some(CalcValue::Percentage(a + b)),
        _ => None, // Incompatible types
    }
}

fn sub_values(a: CalcValue, b: CalcValue) -> Option<CalcValue> {
    match (a, b) {
        (CalcValue::Length(a), CalcValue::Length(b)) => Some(CalcValue::Length(a - b)),
        (CalcValue::Percentage(a), CalcValue::Percentage(b)) => Some(CalcValue::Percentage(a - b)),
        (CalcValue::Number(a), CalcValue::Number(b)) => Some(CalcValue::Number(a - b)),
        (CalcValue::Length(a), CalcValue::Number(b)) => Some(CalcValue::Length(a - b)),
        (CalcValue::Number(a), CalcValue::Length(b)) => Some(CalcValue::Length(a - b)),
        _ => None,
    }
}

fn mul_values(a: CalcValue, b: CalcValue) -> Option<CalcValue> {
    match (a, b) {
        (CalcValue::Length(a), CalcValue::Number(b))
        | (CalcValue::Number(b), CalcValue::Length(a)) => Some(CalcValue::Length(a * b)),
        (CalcValue::Percentage(a), CalcValue::Number(b))
        | (CalcValue::Number(b), CalcValue::Percentage(a)) => Some(CalcValue::Percentage(a * b)),
        (CalcValue::Number(a), CalcValue::Number(b)) => Some(CalcValue::Number(a * b)),
        _ => None,
    }
}

fn div_values(a: CalcValue, b: CalcValue) -> Option<CalcValue> {
    match (a, b) {
        (CalcValue::Length(a), CalcValue::Number(b)) if b != 0.0 => {
            Some(CalcValue::Length(a / b))
        }
        (CalcValue::Percentage(a), CalcValue::Number(b)) if b != 0.0 => {
            Some(CalcValue::Percentage(a / b))
        }
        (CalcValue::Number(a), CalcValue::Number(b)) if b != 0.0 => {
            Some(CalcValue::Number(a / b))
        }
        _ => None,
    }
}

fn negate_value(v: CalcValue) -> CalcValue {
    match v {
        CalcValue::Length(a) => CalcValue::Length(-a),
        CalcValue::Percentage(a) => CalcValue::Percentage(-a),
        CalcValue::Number(a) => CalcValue::Number(-a),
        other => other,
    }
}

/// Try to parse a CSS length string (e.g., `"10px"`) and convert to px.
#[must_use]
pub fn try_parse_length_to_px(raw: &str, ctx: &CalcContext) -> Option<f64> {
    let raw = raw.trim();

    if let Some(num_str) = raw.strip_suffix("px") {
        num_str.parse::<f64>().ok()
    } else if let Some(num_str) = raw.strip_suffix("em") {
        num_str.parse::<f64>().ok().map(|v| v * ctx.font_size)
    } else if let Some(num_str) = raw.strip_suffix("rem") {
        num_str.parse::<f64>().ok().map(|v| v * 16.0)
    } else if let Some(num_str) = raw.strip_suffix("vw") {
        num_str.parse::<f64>().ok().map(|v| v * ctx.viewport_width / 100.0)
    } else if let Some(num_str) = raw.strip_suffix("vh") {
        num_str.parse::<f64>().ok().map(|v| v * ctx.viewport_height / 100.0)
    } else {
        raw.parse::<f64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_ctx() -> CalcContext {
        CalcContext {
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            parent_width: Some(500.0),
            font_size: 16.0,
        }
    }

    #[test]
    fn test_calc_add_same_unit() {
        let result = evaluate_calc("10px + 20px", &default_ctx());
        assert_eq!(result, CalcValue::Length(30.0));
    }

    #[test]
    fn test_calc_sub_same_unit() {
        let result = evaluate_calc("100px - 40px", &default_ctx());
        assert_eq!(result, CalcValue::Length(60.0));
    }

    #[test]
    fn test_calc_mul() {
        let result = evaluate_calc("10px * 2", &default_ctx());
        assert_eq!(result, CalcValue::Length(20.0));
    }

    #[test]
    fn test_calc_div() {
        let result = evaluate_calc("20px / 4", &default_ctx());
        assert_eq!(result, CalcValue::Length(5.0));
    }

    #[test]
    fn test_calc_percentage_add() {
        let result = evaluate_calc("50% + 10%", &CalcContext::default());
        // No parent width → percentages stay as percentages
        assert_eq!(result, CalcValue::Percentage(60.0));
    }

    #[test]
    fn test_calc_percentage_add_with_parent() {
        let result = evaluate_calc("50% + 10%", &default_ctx());
        // 50% of 500 = 250px, 10% of 500 = 50px → 300px
        assert_eq!(result, CalcValue::Length(300.0));
    }

    #[test]
    fn test_calc_percent_with_parent() {
        let result = evaluate_calc("100% - 40px", &default_ctx());
        // 100% of 500px = 500px, minus 40px = 460px
        assert_eq!(result, CalcValue::Length(460.0));
    }

    #[test]
    fn test_calc_vw() {
        let result = evaluate_calc("100vw - 40px", &default_ctx());
        // 100vw = 1920px, minus 40px = 1880px
        assert_eq!(result, CalcValue::Length(1880.0));
    }

    #[test]
    fn test_calc_negative() {
        let result = evaluate_calc("-10px + 20px", &default_ctx());
        assert_eq!(result, CalcValue::Length(10.0));
    }

    #[test]
    fn test_calc_parens() {
        let result = evaluate_calc("(10px + 20px) * 2", &default_ctx());
        assert_eq!(result, CalcValue::Length(60.0));
    }

    #[test]
    fn test_calc_single_value() {
        let result = evaluate_calc("10px", &default_ctx());
        assert_eq!(result, CalcValue::Length(10.0));
    }

    #[test]
    fn test_calc_raw_fallback() {
        let result = evaluate_calc("10px + 10em", &CalcContext::default());
        // 10px + 160px (10em * 16px) — wait, these are different types
        // 10px is Length, 10em should also become Length
        // Actually em gets converted to px in eval_primary, so this should work
        assert_eq!(result, CalcValue::Length(170.0)); // 10 + 10*16
    }
}
