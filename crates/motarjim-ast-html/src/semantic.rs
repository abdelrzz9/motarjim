//! Semantic analysis types for the Motarjim compiler.

use crate::node::NodeId;

/// Represents the severity of an accessibility violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// An error that prevents compilation from succeeding.
    Error,
    /// A warning about a potential issue.
    Warning,
    /// An informational message.
    Info,
    /// A hint for improving code.
    Hint,
    /// An additional note providing context.
    Note,
}

/// Semantic IR node types used for role inference.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum SemanticIr {
    /// The root of the UI tree.
    Root,
    /// A navigation container.
    Navigation,
    /// A navigation bar.
    NavigationBar,
    /// A hero section.
    HeroSection,
    /// A card container.
    Card,
    /// A clickable button.
    Button,
    /// A text element.
    Text,
    /// A heading at the specified level.
    Heading {
        /// The heading level (1–6).
        level: u32,
    },
    /// A paragraph of text.
    Paragraph,
    /// An image element.
    Image,
    /// An icon element.
    Icon,
    /// A text input field.
    Input,
    /// A multi-line text area.
    TextArea,
    /// A dropdown select control.
    Select,
    /// A checkbox control.
    Checkbox,
    /// A radio button control.
    Radio,
    /// A form container.
    Form,
    /// A list container.
    List,
    /// A single list item.
    ListItem,
    /// A table container.
    Table,
    /// A table row.
    TableRow,
    /// A table cell.
    TableCell,
    /// A generic section.
    Section,
    /// An article element.
    Article,
    /// A sidebar or complementary content.
    Aside,
    /// A page footer.
    Footer,
    /// A page header.
    Header,
    /// The main content area.
    Main,
    /// A dialog or modal overlay.
    Dialog,
    /// A tooltip popup.
    Tooltip,
    /// A badge or notification indicator.
    Badge,
    /// A visual divider line.
    Divider,
    /// A spacer element.
    Spacer,
    /// A generic container.
    Container,
    /// A grid container.
    Grid,
    /// A flex container.
    Flex,
    /// A column layout.
    Column,
    /// A row layout.
    Row,
    /// A stack layout.
    Stack,
    /// A scrollable container.
    Scroll,
    /// A lazily-loaded list.
    LazyList,
    /// An icon button.
    IconButton,
    /// A chip/tag element.
    Chip,
    /// An avatar/image circle.
    Avatar,
    /// A progress indicator.
    Progress,
    /// A skeleton loading placeholder.
    Skeleton,
    /// A custom semantic role.
    Custom(smol_str::SmolStr),
}

/// A semantic role assignment with a confidence score.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticRole {
    /// The inferred semantic role.
    pub role: SemanticIr,
    /// The confidence level (0.0–1.0).
    pub confidence: f64,
}

/// Accessibility metadata for a node.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct AccessibilityInfo {
    /// The ARIA role.
    pub role: Option<String>,
    /// The accessible label.
    pub label: Option<String>,
    /// A longer accessible description.
    pub description: Option<String>,
    /// Whether the element is focused.
    pub focused: bool,
    /// Whether the element is hidden from the accessibility tree.
    pub hidden: bool,
    /// The tab index.
    pub tab_index: Option<i32>,
    /// The ARIA level.
    pub aria_level: Option<u32>,
    /// Whether the element is expanded.
    pub aria_expanded: Option<bool>,
    /// The ID of the element this element controls.
    pub aria_controls: Option<String>,
    /// The ARIA live region mode.
    pub aria_live: Option<String>,
    /// Whether the element is busy.
    pub aria_busy: bool,
}

/// An accessibility violation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct A11yViolation {
    /// The numeric violation code.
    pub code: u32,
    /// A human-readable description.
    pub message: String,
    /// The severity of the violation.
    pub severity: Severity,
    /// The ID of the node where the violation was detected.
    pub node_id: NodeId,
    /// An optional suggestion for fixing the violation.
    pub suggestion: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_role() {
        let role = SemanticRole {
            role: SemanticIr::Button,
            confidence: 0.95,
        };
        assert_eq!(role.role, SemanticIr::Button);
        assert!((role.confidence - 0.95).abs() < f64::EPSILON);

        let low = SemanticRole {
            role: SemanticIr::Heading { level: 1 },
            confidence: 0.45,
        };
        assert_eq!(low.role, SemanticIr::Heading { level: 1 });
    }

    #[test]
    fn test_accessibility_info_defaults() {
        let info = AccessibilityInfo {
            role: None,
            label: None,
            description: None,
            focused: false,
            hidden: false,
            tab_index: None,
            aria_level: None,
            aria_expanded: None,
            aria_controls: None,
            aria_live: None,
            aria_busy: false,
        };
        assert!(info.role.is_none() && !info.focused && !info.hidden && !info.aria_busy);
    }

    #[test]
    fn test_accessibility_info_with_aria() {
        let info = AccessibilityInfo {
            role: Some("button".to_string()),
            label: Some("Submit".to_string()),
            description: None,
            focused: true,
            hidden: false,
            tab_index: Some(0),
            aria_level: None,
            aria_expanded: None,
            aria_controls: Some("form-1".to_string()),
            aria_live: Some("polite".to_string()),
            aria_busy: false,
        };
        assert_eq!(info.role.as_deref(), Some("button"));
        assert_eq!(info.label.as_deref(), Some("Submit"));
        assert!(info.focused && info.tab_index == Some(0));
    }

    #[test]
    fn test_a11y_violation() {
        let v = A11yViolation {
            code: 301,
            message: "Image is missing alt text".to_string(),
            severity: Severity::Warning,
            node_id: NodeId(5),
            suggestion: Some("Add alt text".to_string()),
        };
        assert_eq!(v.code, 301);
        assert_eq!(v.severity, Severity::Warning);
        assert_eq!(v.node_id, NodeId(5));

        let v2 = A11yViolation {
            code: 101,
            message: "No label".to_string(),
            severity: Severity::Error,
            node_id: NodeId(10),
            suggestion: None,
        };
        assert_eq!(v2.code, 101);
        assert_eq!(v2.severity, Severity::Error);
        assert!(v2.suggestion.is_none());
    }
}
