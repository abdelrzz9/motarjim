//! `CssValue` and related types for the Motarjim compiler.
//!
//! These types represent parsed CSS property values in a structured form
//! that is backend-agnostic and independent of any external parser.

use smol_str::SmolStr;

/// A wrapper around `f64` that implements `Eq`, `Hash`, and `Ord` by treating
/// all NaN values as equal and using total ordering.
///
/// This is a simplified version of `ordered_float::OrderedFloat` without the
/// external dependency.
#[derive(Debug, Clone, Copy)]
pub struct CssNumber(pub f64);

impl PartialEq for CssNumber {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for CssNumber {}

impl std::hash::Hash for CssNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl PartialOrd for CssNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CssNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.to_bits().cmp(&other.0.to_bits())
    }
}

impl From<f64> for CssNumber {
    fn from(v: f64) -> Self {
        Self(v)
    }
}

impl From<CssNumber> for f64 {
    fn from(v: CssNumber) -> Self {
        v.0
    }
}

impl std::fmt::Display for CssNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A parsed CSS property value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum CssValue {
    /// A keyword value (e.g. `red`, `block`, `flex`).
    Keyword(SmolStr),
    /// A numeric value (e.g. `1`, `0.5`).
    Number(CssNumber),
    /// A length value with a unit (e.g. `16px`, `2em`).
    Length(CssNumber, CssUnit),
    /// A percentage value (e.g. `50%`).
    Percentage(CssNumber),
    /// A color value.
    Color {
        /// Red channel (0–255).
        r: u8,
        /// Green channel (0–255).
        g: u8,
        /// Blue channel (0–255).
        b: u8,
        /// Alpha channel (0.0–1.0).
        a: CssNumber,
        /// The color space (e.g. `"srgb"`, `"display-p3"`).
        color_space: SmolStr,
    },
    /// A URL value (e.g. `url("...")`).
    Url(SmolStr),
    /// A quoted string value.
    CssString(SmolStr),
    /// A function call (e.g. `calc()`, `var()`, `linear-gradient()`).
    Function(CssFunction),
    /// A comma-separated list of values.
    CommaSeparatedList(Vec<CssValue>),
    /// A space-separated list of values.
    SpaceSeparatedList(Vec<CssValue>),
    /// A CSS variable reference (e.g. `var(--x)`).
    Variable(SmolStr),
    /// The `none` keyword.
    None,
    /// The `auto` keyword.
    Auto,
    /// The `inherit` keyword.
    Inherit,
    /// The `initial` keyword.
    Initial,
    /// The `unset` keyword.
    Unset,
    /// An identifier value not covered by other variants.
    Ident(SmolStr),
}

/// A CSS function call with name and arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct CssFunction {
    /// The function name (e.g. `"calc"`, `"var"`, `"linear-gradient"`).
    pub name: SmolStr,
    /// The function arguments.
    pub arguments: Vec<CssValue>,
}

/// A CSS length unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssUnit {
    /// Pixels.
    Px,
    /// Ems (relative to parent font size).
    Em,
    /// Rems (relative to root font size).
    Rem,
    /// Viewport width units.
    Vw,
    /// Viewport height units.
    Vh,
    /// Viewport minimum units.
    Vmin,
    /// Viewport maximum units.
    Vmax,
    /// Degrees (for angles).
    Deg,
    /// Radians (for angles).
    Rad,
    /// Gradians (for angles).
    Grad,
    /// Turns (for angles).
    Turn,
    /// Seconds (for time).
    S,
    /// Milliseconds (for time).
    Ms,
    /// Fractional units (CSS Grid).
    Fr,
    /// `ch` unit (advance measure of "0").
    Ch,
    /// `ex` unit (x-height of font).
    Ex,
    /// Centimeters.
    Cm,
    /// Millimeters.
    Mm,
    /// Inches.
    In,
    /// Points.
    Pt,
    /// Picas.
    Pc,
    /// Viewport width (large).
    VwLarge,
    /// Viewport height (large).
    VhLarge,
    /// Viewport width (small).
    VwSmall,
    /// Viewport height (small).
    VhSmall,
    /// Viewport width (dynamic).
    VwDynamic,
    /// Viewport height (dynamic).
    VhDynamic,
}

impl std::fmt::Display for CssUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Px => write!(f, "px"),
            Self::Em => write!(f, "em"),
            Self::Rem => write!(f, "rem"),
            Self::Vw => write!(f, "vw"),
            Self::Vh => write!(f, "vh"),
            Self::Vmin => write!(f, "vmin"),
            Self::Vmax => write!(f, "vmax"),
            Self::Deg => write!(f, "deg"),
            Self::Rad => write!(f, "rad"),
            Self::Grad => write!(f, "grad"),
            Self::Turn => write!(f, "turn"),
            Self::S => write!(f, "s"),
            Self::Ms => write!(f, "ms"),
            Self::Fr => write!(f, "fr"),
            Self::Ch => write!(f, "ch"),
            Self::Ex => write!(f, "ex"),
            Self::Cm => write!(f, "cm"),
            Self::Mm => write!(f, "mm"),
            Self::In => write!(f, "in"),
            Self::Pt => write!(f, "pt"),
            Self::Pc => write!(f, "pc"),
            Self::VwLarge => write!(f, "lvw"),
            Self::VhLarge => write!(f, "lvh"),
            Self::VwSmall => write!(f, "svw"),
            Self::VhSmall => write!(f, "svh"),
            Self::VwDynamic => write!(f, "dvw"),
            Self::VhDynamic => write!(f, "dvh"),
        }
    }
}

impl std::str::FromStr for CssUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "px" => Ok(Self::Px),
            "em" => Ok(Self::Em),
            "rem" => Ok(Self::Rem),
            "vw" => Ok(Self::Vw),
            "vh" => Ok(Self::Vh),
            "vmin" => Ok(Self::Vmin),
            "vmax" => Ok(Self::Vmax),
            "deg" => Ok(Self::Deg),
            "rad" => Ok(Self::Rad),
            "grad" => Ok(Self::Grad),
            "turn" => Ok(Self::Turn),
            "s" => Ok(Self::S),
            "ms" => Ok(Self::Ms),
            "fr" => Ok(Self::Fr),
            "ch" => Ok(Self::Ch),
            "ex" => Ok(Self::Ex),
            "cm" => Ok(Self::Cm),
            "mm" => Ok(Self::Mm),
            "in" => Ok(Self::In),
            "pt" => Ok(Self::Pt),
            "pc" => Ok(Self::Pc),
            "lvw" => Ok(Self::VwLarge),
            "lvh" => Ok(Self::VhLarge),
            "svw" => Ok(Self::VwSmall),
            "svh" => Ok(Self::VhSmall),
            "dvw" => Ok(Self::VwDynamic),
            "dvh" => Ok(Self::VhDynamic),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_unit_display_and_parse() {
        for unit in &[
            CssUnit::Px,
            CssUnit::Em,
            CssUnit::Rem,
            CssUnit::Vw,
            CssUnit::Vh,
            CssUnit::Deg,
            CssUnit::Rad,
            CssUnit::S,
            CssUnit::Ms,
            CssUnit::Fr,
        ] {
            let s = unit.to_string();
            let parsed: CssUnit = s.parse().unwrap();
            assert_eq!(*unit, parsed, "round-trip failed for {unit:?}");
        }
    }

    #[test]
    fn test_css_value_variants() {
        let kw = CssValue::Keyword(SmolStr::new_inline("red"));
        assert_eq!(kw, CssValue::Keyword(SmolStr::new_inline("red")));

        let num = CssValue::Number(CssNumber(42.0));
        assert_eq!(num, CssValue::Number(CssNumber(42.0)));

        let len = CssValue::Length(CssNumber(16.0), CssUnit::Px);
        assert_eq!(len, CssValue::Length(CssNumber(16.0), CssUnit::Px));

        let color = CssValue::Color {
            r: 255,
            g: 0,
            b: 0,
            a: CssNumber(1.0),
            color_space: SmolStr::new_inline("srgb"),
        };
        assert!(matches!(color, CssValue::Color { r: 255, .. }));

        let func = CssFunction {
            name: SmolStr::new_inline("calc"),
            arguments: vec![
                CssValue::Percentage(CssNumber(100.0)),
                CssValue::Keyword(SmolStr::new_inline("-")),
                CssValue::Length(CssNumber(20.0), CssUnit::Px),
            ],
        };
        assert!(matches!(func.name.as_str(), "calc"));
    }

    #[test]
    fn test_css_number_eq() {
        assert_eq!(CssNumber(1.0), CssNumber(1.0));
        assert_eq!(CssNumber(f64::NAN), CssNumber(f64::NAN));
        assert_ne!(CssNumber(1.0), CssNumber(2.0));
    }
}
