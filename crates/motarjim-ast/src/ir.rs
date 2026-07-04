#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Intermediate representation (IR) types for the Motarjim compiler.

use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::html::NodeId;
use crate::style::ComputedStyle;

/// The main IR node, wrapping semantic, layout, and target info with a computed style.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct IrNode {
    /// The unique node identifier.
    pub id: NodeId,
    /// The semantic IR.
    pub semantic: SemanticIr,
    /// The layout IR.
    pub layout: LayoutIr,
    /// The target IR.
    pub target: TargetIr,
    /// The resolved computed style.
    pub computed_style: ComputedStyle,
    /// The IDs of this node's children.
    pub children: SmallVec<[NodeId; 4]>,
    /// The ID of this node's parent, if any.
    pub parent: Option<NodeId>,
}

/// The full IR tree.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct IrTree {
    /// All IR nodes in the tree.
    pub nodes: Vec<IrNode>,
    /// The ID of the root node.
    pub root_id: NodeId,
    /// Target hints for the entire tree.
    pub target_hints: Vec<TargetHint>,
}

/// Semantic IR node types.
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
    Custom(SmolStr),
}

/// Layout IR node types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum LayoutIr {
    /// A flex container with row direction.
    FlexRow,
    /// A flex container with column direction.
    FlexColumn,
    /// A CSS Grid container.
    Grid,
    /// A stack layout (block flow).
    Stack,
    /// A z-axis stack (layered).
    ZStack,
    /// A scrollable container.
    Scroll,
    /// A lazily-loaded list.
    LazyList,
    /// An absolutely positioned node.
    Absolute,
    /// A relatively positioned node.
    Relative,
    /// A statically positioned node.
    Static,
    /// A sticky positioned node.
    Sticky,
    /// A fixed positioned node.
    Fixed,
    /// A normal flow layout.
    Flow,
    /// An inline layout.
    Inline,
    /// An inline-block layout.
    InlineBlock,
    /// A table layout.
    Table,
    /// No specific layout.
    None,
}

/// Target-specific widget hints for code generation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum TargetIr {
    /// A Flutter/Dart widget.
    Flutter {
        /// The Flutter widget class name.
        widget: SmolStr,
        /// Key-value properties for the widget constructor.
        properties: Vec<(SmolStr, String)>,
    },
    /// A Jetpack Compose composable function.
    Compose {
        /// The composable function name.
        composable: SmolStr,
        /// Key-value properties for the composable.
        properties: Vec<(SmolStr, String)>,
    },
    /// A SwiftUI view struct.
    SwiftUI {
        /// The SwiftUI view type name.
        view: SmolStr,
        /// Key-value properties for the view.
        properties: Vec<(SmolStr, String)>,
    },
    /// A generic target platform node.
    Generic {
        /// The platform name.
        platform: SmolStr,
        /// The node name on this platform.
        node: SmolStr,
    },
}

/// Target hints for the IR tree.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct TargetHint {
    /// The target platform name.
    pub target: SmolStr,
    /// The type of hint.
    pub hint_type: HintType,
    /// The hint value.
    pub value: String,
}

/// The type of a target hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum HintType {
    /// A widget name hint.
    Widget,
    /// A modifier hint.
    Modifier,
    /// An import requirement hint.
    Import,
    /// A property hint.
    Property,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::ComputedStyle;

    fn make_ir_node(id: u32, semantic: SemanticIr, layout: LayoutIr) -> IrNode {
        IrNode {
            id: NodeId(id),
            semantic,
            layout,
            target: TargetIr::Generic {
                platform: SmolStr::new_inline("flutter"),
                node: SmolStr::new_inline("Container"),
            },
            computed_style: ComputedStyle::default(),
            children: SmallVec::new(),
            parent: None,
        }
    }

    #[test]
    fn test_ir_node() {
        let node = make_ir_node(0, SemanticIr::Root, LayoutIr::FlexColumn);
        assert_eq!(node.id, NodeId(0));
        assert_eq!(node.semantic, SemanticIr::Root);
        assert_eq!(node.layout, LayoutIr::FlexColumn);
    }

    #[test]
    fn test_ir_tree() {
        let tree = IrTree {
            nodes: vec![make_ir_node(0, SemanticIr::Root, LayoutIr::FlexColumn)],
            root_id: NodeId(0),
            target_hints: vec![],
        };
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.root_id, NodeId(0));
    }

    #[test]
    fn test_semantic_ir_variants() {
        assert!(matches!(SemanticIr::Button, SemanticIr::Button));
        assert!(matches!(
            SemanticIr::Heading { level: 2 },
            SemanticIr::Heading { level: 2 }
        ));
        assert!(matches!(
            SemanticIr::Custom(SmolStr::new_inline("carousel")),
            SemanticIr::Custom(_)
        ));
    }

    #[test]
    fn test_layout_ir_variants() {
        assert!(matches!(LayoutIr::FlexRow, LayoutIr::FlexRow));
        assert!(matches!(LayoutIr::Grid, LayoutIr::Grid));
        assert!(matches!(LayoutIr::None, LayoutIr::None));
    }

    #[test]
    fn test_target_ir_variants() {
        assert!(matches!(
            TargetIr::Flutter {
                widget: SmolStr::new_inline("Row"),
                properties: vec![]
            },
            TargetIr::Flutter { .. }
        ));
        assert!(matches!(
            TargetIr::Compose {
                composable: SmolStr::new_inline("Row"),
                properties: vec![]
            },
            TargetIr::Compose { .. }
        ));
        assert!(matches!(
            TargetIr::SwiftUI {
                view: SmolStr::new_inline("HStack"),
                properties: vec![]
            },
            TargetIr::SwiftUI { .. }
        ));
    }

    #[test]
    fn test_target_hint() {
        let hint = TargetHint {
            target: SmolStr::new_inline("flutter"),
            hint_type: HintType::Widget,
            value: "Scaffold".to_string(),
        };
        assert_eq!(hint.target.as_str(), "flutter");
        assert_eq!(hint.hint_type, HintType::Widget);
        assert_eq!(hint.value, "Scaffold");
    }

    #[test]
    fn test_ir_node_parent_child() {
        let mut parent = make_ir_node(0, SemanticIr::Root, LayoutIr::FlexColumn);
        parent.children.push(NodeId(1));
        let child = make_ir_node(1, SemanticIr::Button, LayoutIr::Static);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(child.parent, None);
    }
}
