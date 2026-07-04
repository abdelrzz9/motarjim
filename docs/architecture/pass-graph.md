# Optimization Pass Dependency Graph

```mermaid
graph TD
    RemoveEmptyNodes[[RemoveEmptyNodes]]
    MergeAdjacentText[[MergeAdjacentText]]
    CollapseWhitespace[[CollapseWhitespace]]
    RemoveUnusedStyles[[RemoveUnusedStyles]]
    FlattenNestedContainers[[FlattenNestedContainers]]
    InlineConstantValues[[InlineConstantValues]]

    CollapseWhitespace --> MergeAdjacentText
    RemoveEmptyNodes --> FlattenNestedContainers
    RemoveEmptyNodes --> InlineConstantValues
    FlattenNestedContainers --> InlineConstantValues
```

## Pass Details

| Pass | Prerequisites | Invalidates |
|------|--------------|-------------|
| RemoveEmptyNodes | none | RemoveEmptyNodes, MergeAdjacentText, FlattenNestedContainers, InlineConstantValues |
| MergeAdjacentText | CollapseWhitespace | RemoveEmptyNodes, MergeAdjacentText |
| CollapseWhitespace | none | MergeAdjacentText, CollapseWhitespace |
| RemoveUnusedStyles | none | RemoveUnusedStyles |
| FlattenNestedContainers | RemoveEmptyNodes | FlattenNestedContainers, InlineConstantValues |
| InlineConstantValues | RemoveEmptyNodes, FlattenNestedContainers | InlineConstantValues |

