//! CSS stylesheet AST types for the Motarjim compiler.

use smallvec::SmallVec;
use smol_str::SmolStr;

/// A parsed CSS stylesheet.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct CssStylesheet {
    /// The list of CSS rules in this stylesheet.
    pub rules: Vec<CssRule>,
    /// The optional source file path.
    pub source_path: Option<String>,
}

/// A single CSS rule, which can be a style rule or an at-rule.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum CssRule {
    /// A standard style rule with selectors and declarations.
    Style(StyleRule),
    /// A `@media` rule.
    Media(MediaRule),
    /// A `@font-face` rule.
    FontFace(FontFaceRule),
    /// A `@keyframes` rule.
    Keyframes(KeyframesRule),
    /// A `@import` rule.
    Import(ImportRule),
    /// A `@charset` rule.
    Charset(CharsetRule),
    /// A `@namespace` rule.
    Namespace(NamespaceRule),
    /// A `@supports` rule.
    Supports(SupportsRule),
    /// A `@page` rule.
    Page(PageRule),
    /// A generic at-rule.
    Other(AtRule),
}

/// A standard style rule containing selectors and declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct StyleRule {
    /// The selectors for this rule.
    pub selectors: Vec<crate::selector::Selector>,
    /// The declarations in this rule.
    pub declarations: SmallVec<[Declaration; 4]>,
}

/// A CSS declaration (a property-value pair).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Declaration {
    /// The CSS property name.
    pub property: SmolStr,
    /// The raw CSS value string.
    pub value: String,
    /// Whether this declaration is marked `!important`.
    pub important: bool,
}

/// A `@media` rule containing a media query and nested rules.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MediaRule {
    /// The media query conditions.
    pub query: MediaQuery,
    /// The rules nested inside the `@media` block.
    pub rules: Vec<CssRule>,
}

/// A `@font-face` rule defining a custom font.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct FontFaceRule {
    /// The declarations inside the `@font-face` block.
    pub declarations: SmallVec<[Declaration; 4]>,
}

/// A `@keyframes` rule defining an animation sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyframesRule {
    /// The animation name.
    pub name: SmolStr,
    /// The keyframe blocks in the animation.
    pub keyframes: Vec<Keyframe>,
}

/// A single keyframe block within a `@keyframes` rule.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Keyframe {
    /// The keyframe selectors.
    pub selectors: SmallVec<[SmolStr; 2]>,
    /// The declarations for this keyframe.
    pub declarations: SmallVec<[Declaration; 4]>,
}

/// A `@import` rule for importing an external stylesheet.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportRule {
    /// The URL of the imported stylesheet.
    pub url: SmolStr,
    /// An optional media query restricting the import.
    pub media: Option<MediaQuery>,
}

/// A `@charset` rule specifying the stylesheet character encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct CharsetRule {
    /// The character encoding name.
    pub encoding: SmolStr,
}

/// A `@namespace` rule declaring an XML namespace prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct NamespaceRule {
    /// An optional namespace prefix.
    pub prefix: Option<SmolStr>,
    /// The namespace URL.
    pub url: SmolStr,
}

/// A `@supports` rule for feature-conditional CSS.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct SupportsRule {
    /// The raw supports condition string.
    pub condition: String,
    /// The rules nested inside the `@supports` block.
    pub rules: Vec<CssRule>,
}

/// A `@page` rule for paged media styling.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct PageRule {
    /// An optional page pseudo-class.
    pub pseudo: Option<SmolStr>,
    /// The declarations inside the `@page` block.
    pub declarations: SmallVec<[Declaration; 4]>,
}

/// A generic at-rule that doesn't match a known type.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct AtRule {
    /// The at-rule name.
    pub name: SmolStr,
    /// The prelude (between name and block).
    pub prelude: String,
    /// The block content, if present.
    pub block: Option<String>,
}

/// A parsed media query consisting of one or more conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MediaQuery {
    /// The media conditions that make up this query.
    pub conditions: Vec<MediaCondition>,
}

/// A single media query condition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MediaCondition {
    /// Matches all media types.
    All,
    /// Matches the `print` media type.
    Print,
    /// Matches the `screen` media type.
    Screen,
    /// Matches the `speech` media type.
    Speech,
    /// Matches only the inner condition.
    Only(Box<Self>),
    /// Matches when the inner condition is not met.
    Not(Box<Self>),
    /// Matches when all inner conditions are met (AND).
    And(Vec<Self>),
    /// Matches when any inner condition is met (OR).
    Or(Vec<Self>),
    /// A media feature with a name and optional value.
    Feature {
        /// The feature name.
        name: SmolStr,
        /// The feature value, if present.
        value: Option<String>,
    },
    /// A `min-width` media feature.
    MinWidth(String),
    /// A `max-width` media feature.
    MaxWidth(String),
    /// A `min-height` media feature.
    MinHeight(String),
    /// A `max-height` media feature.
    MaxHeight(String),
    /// An `orientation` media feature.
    Orientation(String),
    /// A `prefers-color-scheme` media feature.
    PrefersColorScheme(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declaration() {
        let decl = Declaration {
            property: SmolStr::new_inline("color"),
            value: "red".to_string(),
            important: false,
        };
        assert_eq!(decl.property.as_str(), "color");
        assert_eq!(decl.value, "red");
        assert!(!decl.important);

        let imp = Declaration {
            property: SmolStr::new_inline("color"),
            value: "blue".to_string(),
            important: true,
        };
        assert!(imp.important);
    }

    #[test]
    fn test_style_rule() {
        let rule = StyleRule {
            selectors: Vec::new(),
            declarations: SmallVec::new(),
        };
        assert!(rule.selectors.is_empty() && rule.declarations.is_empty());
    }

    #[test]
    fn test_media_query() {
        let mq = MediaQuery {
            conditions: vec![MediaCondition::All],
        };
        assert_eq!(mq.conditions.len(), 1);
    }

    #[test]
    fn test_css_rule_variants() {
        assert!(matches!(
            CssRule::Style(StyleRule {
                selectors: Vec::new(),
                declarations: SmallVec::new()
            }),
            CssRule::Style(_)
        ));
        assert!(matches!(
            CssRule::Media(MediaRule {
                query: MediaQuery {
                    conditions: vec![MediaCondition::Screen]
                },
                rules: Vec::new()
            }),
            CssRule::Media(_)
        ));
        assert!(matches!(
            CssRule::FontFace(FontFaceRule {
                declarations: SmallVec::new()
            }),
            CssRule::FontFace(_)
        ));
    }

    #[test]
    fn test_keyframes() {
        let rule = KeyframesRule {
            name: SmolStr::new_inline("fade-in"),
            keyframes: vec![Keyframe {
                selectors: smallvec::smallvec![SmolStr::new_inline("from")],
                declarations: smallvec::smallvec![Declaration {
                    property: SmolStr::new_inline("opacity"),
                    value: "0".to_string(),
                    important: false,
                }],
            }],
        };
        assert_eq!(rule.name.as_str(), "fade-in");
        assert_eq!(rule.keyframes.len(), 1);
    }

    #[test]
    fn test_import_rule() {
        let import = ImportRule {
            url: SmolStr::from("https://fonts.googleapis.com/css"),
            media: None,
        };
        assert_eq!(import.url.as_str(), "https://fonts.googleapis.com/css");
        assert!(import.media.is_none());
    }

    #[test]
    fn test_at_rule() {
        let at = AtRule {
            name: SmolStr::new_inline("viewport"),
            prelude: String::new(),
            block: Some("width=device-width".to_string()),
        };
        assert_eq!(at.name.as_str(), "viewport");
        assert!(at.block.is_some());
    }
}
