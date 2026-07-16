#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::all)]
#![allow(clippy::pedantic, clippy::nursery)]

//! Serialization helpers for the Motarjim compiler.
//!
//! Provides JSON serialization and deserialization for the compiler's
//! intermediate representation (IR) and configuration types.
//!
//! # Example
//!
//! ```rust
//! use motarjim_serialize::ir_json;
//! use motarjim_ast::ir::IrTree;
//! use motarjim_ast::NodeId;
//!
//! let tree = IrTree {
//!     nodes: vec![],
//!     root_id: NodeId(0),
//!     target_hints: vec![],
//! };
//! let json = ir_json::to_string(&tree).unwrap();
//! assert!(!json.is_empty());
//! ```

/// JSON serialization for the IR tree.
pub mod ir_json {
    use motarjim_ast::ir::{IrNode, IrTree};

    /// Serializes an `IrTree` to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails.
    pub fn to_string(tree: &IrTree) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(tree)
    }

    /// Serializes an `IrTree` to a JSON byte vector.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails.
    pub fn to_vec(tree: &IrTree) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(tree)
    }

    /// Deserializes an `IrTree` from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if deserialization fails.
    pub fn from_str(s: &str) -> Result<IrTree, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Deserializes an `IrTree` from a JSON byte slice.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if deserialization fails.
    pub fn from_slice(v: &[u8]) -> Result<IrTree, serde_json::Error> {
        serde_json::from_slice(v)
    }

    /// Serializes a single `IrNode` to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails.
    pub fn node_to_string(node: &IrNode) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(node)
    }
}

/// JSON serialization for the Motarjim configuration.
pub mod config_json {
    use std::collections::HashMap;

    /// A serializable configuration structure.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Config {
        /// The target platform.
        pub target: Option<String>,
        /// The input HTML file path.
        pub input: Option<String>,
        /// The input CSS file path.
        pub css: Option<String>,
        /// The output directory.
        pub output: Option<String>,
        /// Watch mode flag.
        pub watch: Option<bool>,
        /// Format output flag.
        pub format: Option<bool>,
        /// Custom settings.
        #[serde(default)]
        pub settings: HashMap<String, String>,
    }

    /// Parses a configuration from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if parsing fails.
    pub fn from_str(s: &str) -> Result<Config, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Serializes a configuration to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails.
    pub fn to_string(config: &Config) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(config)
    }
}

/// Binary serialization helpers (MessagePack-like compact format).
pub mod binary {
    use motarjim_ast::ir::IrTree;

    /// Serializes an `IrTree` to a compact binary format.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if serialization fails.
    pub fn to_vec(tree: &IrTree) -> Result<Vec<u8>, serde_json::Error> {
        // For now, use JSON as the binary format. Replace with MessagePack later.
        serde_json::to_vec(tree)
    }

    /// Deserializes an `IrTree` from a compact binary format.
    ///
    /// # Errors
    ///
    /// Returns a `serde_json::Error` if deserialization fails.
    pub fn from_slice(v: &[u8]) -> Result<IrTree, serde_json::Error> {
        serde_json::from_slice(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motarjim_ast::ir::{IrNode, IrTree, LayoutIr, SemanticIr, TargetIr};
    use motarjim_ast::NodeId;
    use motarjim_ast_html::ComputedStyle;

    fn sample_tree() -> IrTree {
        IrTree {
            nodes: vec![IrNode {
                id: NodeId(0),
                semantic: SemanticIr::Container,
                layout: LayoutIr::Stack,
                target: TargetIr::Generic {
                    platform: smol_str::SmolStr::new_inline("generic"),
                    node: smol_str::SmolStr::new_inline("Container"),
                },
                computed_style: ComputedStyle::default(),
                children: smallvec::smallvec![],
                parent: None,
                text: None,
                responsive: Vec::new(),
                events: Vec::new(),
                text_direction: None,
            }],
            root_id: NodeId(0),
            target_hints: vec![],
        }
    }

    #[test]
    fn test_ir_json_roundtrip() {
        let tree = sample_tree();
        let json = ir_json::to_string(&tree).unwrap();
        let deserialized: IrTree = ir_json::from_str(&json).unwrap();
        assert_eq!(deserialized.root_id, NodeId(0));
        assert_eq!(deserialized.nodes.len(), 1);
    }

    #[test]
    fn test_ir_json_to_vec() {
        let tree = sample_tree();
        let vec = ir_json::to_vec(&tree).unwrap();
        assert!(!vec.is_empty());
    }

    #[test]
    fn test_ir_json_from_slice() {
        let tree = sample_tree();
        let vec = ir_json::to_vec(&tree).unwrap();
        let deserialized = ir_json::from_slice(&vec).unwrap();
        assert_eq!(deserialized.nodes.len(), 1);
    }

    #[test]
    fn test_ir_json_node() {
        let node = IrNode {
            id: NodeId(42),
            semantic: SemanticIr::Button,
            layout: LayoutIr::FlexRow,
            target: TargetIr::Flutter {
                widget: smol_str::SmolStr::new_inline("Container"),
                properties: vec![],
            },
            computed_style: ComputedStyle::default(),
            children: smallvec::smallvec![],
            parent: None,
            text: None,
            responsive: Vec::new(),
            events: Vec::new(),
            text_direction: None,
        };
        let json = ir_json::node_to_string(&node).unwrap();
        assert!(json.contains("Button"));
    }

    #[test]
    fn test_config_json_roundtrip() {
        let config = config_json::Config {
            target: Some("flutter".to_string()),
            input: Some("index.html".to_string()),
            css: Some("style.css".to_string()),
            output: Some("output".to_string()),
            watch: Some(true),
            format: Some(false),
            settings: std::collections::HashMap::new(),
        };
        let json = config_json::to_string(&config).unwrap();
        let deserialized = config_json::from_str(&json).unwrap();
        assert_eq!(deserialized.target, Some("flutter".to_string()));
        assert_eq!(deserialized.input, Some("index.html".to_string()));
    }

    #[test]
    fn test_binary_roundtrip() {
        let tree = sample_tree();
        let bytes = binary::to_vec(&tree).unwrap();
        let deserialized = binary::from_slice(&bytes).unwrap();
        assert_eq!(deserialized.nodes.len(), 1);
    }

    #[test]
    fn test_empty_tree() {
        let tree = IrTree {
            nodes: vec![],
            root_id: NodeId(0),
            target_hints: vec![],
        };
        let json = ir_json::to_string(&tree).unwrap();
        let deserialized = ir_json::from_str(&json).unwrap();
        assert!(deserialized.nodes.is_empty());
    }
}
