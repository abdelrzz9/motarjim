use crate::*;

/// Infers layout strategies from CSS computed styles.
#[derive(Debug, Clone)]
pub struct LayoutInferrer {}

impl LayoutInferrer {
    /// Creates a new layout inferrer.
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Infers the layout strategy from a node's computed style.
    #[must_use]
    pub fn infer(&self, style: &ComputedStyle) -> LayoutIr {
        if style.display == DisplayType::None {
            return LayoutIr::None;
        }

        match style.position {
            PositionType::Absolute => return LayoutIr::Absolute,
            PositionType::Fixed => return LayoutIr::Fixed,
            PositionType::Sticky => return LayoutIr::Sticky,
            PositionType::Relative | PositionType::Static => {}
        }

        if style.display == DisplayType::Flex {
            return match style.flex_direction {
                Some(FlexDirection::Row) | Some(FlexDirection::RowReverse) => LayoutIr::FlexRow,
                Some(FlexDirection::Column) | Some(FlexDirection::ColumnReverse) => {
                    LayoutIr::FlexColumn
                }
                None => LayoutIr::FlexRow,
            };
        }

        if style.display == DisplayType::Grid {
            return LayoutIr::Grid;
        }

        if let Some(ref overflow) = style.overflow {
            if *overflow == Overflow::Scroll || *overflow == Overflow::Auto {
                return LayoutIr::Scroll;
            }
        }

        match style.display {
            DisplayType::Block | DisplayType::FlowRoot => LayoutIr::Stack,
            DisplayType::Flow => LayoutIr::Flow,
            DisplayType::Inline => LayoutIr::Inline,
            DisplayType::InlineBlock => LayoutIr::InlineBlock,
            DisplayType::Table | DisplayType::TableRow | DisplayType::TableCell => LayoutIr::Table,
            DisplayType::ListItem => LayoutIr::Stack,
            DisplayType::Contents => LayoutIr::None,
            DisplayType::Flex | DisplayType::Grid | DisplayType::None => LayoutIr::None,
        }
    }
}

impl Default for LayoutInferrer {
    fn default() -> Self {
        Self::new()
    }
}
