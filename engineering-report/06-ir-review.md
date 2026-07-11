# Intermediate Representation (IR) Review

## Current IR Structure

### ast-ir types (`motarjim-ast-ir/src/`)

```rust
pub struct IrTree {
    pub nodes: Vec<IrNode>,
    pub root_id: NodeId,
    pub target_hints: Vec<TargetHint>,
}

pub struct IrNode {
    pub id: NodeId,
    pub semantic: SemanticIr,       // 41 variants
    pub layout: LayoutIr,           // 17 variants
    pub target: TargetIr,           // 4 variants
    pub computed_style: ComputedStyle,
    pub children: SmallVec<[NodeId; 4]>,
    pub parent: Option<NodeId>,
    pub text: Option<String>,
}
```

### SemanticIr (41 variants)

```
Root, Navigation, NavigationBar, HeroSection, Card, Button, Text,
Heading { level: u32 }, Paragraph, Image, Icon, Input, TextArea,
Select, Checkbox, Radio, Form, List, ListItem, Table, TableRow,
TableCell, Section, Article, Aside, Footer, Header, Main, Dialog,
Tooltip, Badge, Divider, Spacer, Container, Grid, Flex, Column,
Row, Stack, Scroll, LazyList, IconButton, Chip, Avatar, Progress,
Skeleton, Custom(SmolStr)
```

### LayoutIr (17 variants)

```
FlexRow, FlexColumn, Grid, Stack, ZStack, Scroll, LazyList,
Absolute, Relative, Static, Sticky, Fixed, Flow, Inline,
InlineBlock, Table, None
```

### TargetIr (4 variants)

```
Flutter { widget: SmolStr, properties: Vec<(SmolStr, String)> },
Compose { composable: SmolStr, properties: Vec<(SmolStr, String)> },
SwiftUI { view: SmolStr, properties: Vec<(SmolStr, String)> },
Generic { platform: SmolStr, node: SmolStr }
```

### TargetHint

```rust
pub struct TargetHint {
    pub target: SmolStr,      // e.g., "responsive", "accessibility"
    pub hint_type: HintType,  // Widget | Modifier | Import | Property
    pub value: String,
}
```

## Assessment

### Is the IR expressive enough for current targets?

| Target | Assessment |
|--------|------------|
| **Flutter** | 🟡 Partial — Has Row/Column/Stack/ListView/Grid layout, but missing: Expanded/Flexible semantics, SizedBox, AspectRatio, FittedBox, Align, Center, ConstrainedBox, IntrinsicWidth/Height, TickerMode |
| **Compose** | 🟡 Partial — Similar coverage, missing: weight/weightIn, requiredWidth/Height, IntrinsicSize, AspectRatio, zIndex, offset, clip, shadow (all Compose Modifier properties) |
| **SwiftUI** | 🟡 Partial — Missing: LazyVStack/LazyHStack, GeometryReader, PreferenceKey, matchedGeometryEffect, alignmentGuide, fixedSize, layoutPriority, aspectRatio |

### Is the IR expressive enough for future targets?

| Target | Assessment |
|--------|------------|
| **React Native** | 🔴 No — Missing: numeric flex, onPress/onLayout event data, style merger for inline styles, Text nesting semantics |
| **Qt/QML** | 🔴 No — Missing: Layout.stretch, Layout.fillWidth/Height, signal/slot, property bindings, anchors |
| **Tauri/Web** | 🔴 No — Missing: preserve CSS class names, preserve HTML structure for hydration, div/span differentiation |
| **UIKit** | 🔴 No — Missing: Auto Layout constraints, NSLayoutAnchor, UIStackView distribution, content hugging/compression resistance |
| **MAUI/.NET** | 🔴 No — Missing: Grid row/column definitions, VerticalStackLayout/HorizontalStackLayout, AbsoluteLayout bounds |

### Should the IR be split?

**Yes.** The current single `IrNode` with three layers (Semantic, Layout, Target) is a good start but insufficient for a production compiler. The `ARCHITECTURE-v2.md` document proposes HIR/MIR/LIR — this is the correct direction.

### Proposed IR Architecture

```
Semantic IR (HIR - High-Level IR)
├── Pure tag/role inference
├── Platform-neutral
├── Stable across backends
├── Contains: semantic role, ARIA metadata, inferred intent
└── Example: <nav class="tabs"> → Navigation with .tabs variant

     ↓ Lowering pass

Layout IR (MIR - Mid-Level IR)
├── Box tree with explicit layout strategy
├── Flex attributes (grow, shrink, basis, align)
├── Grid track definitions (parsed columns/rows)
├── Positioning (absolute offsets, z-index)
├── Overflow behavior
├── Platform-neutral
└── Example: display:flex; flex-direction:row; gap:16px → FlexRow(gap: Px(16))

     ↓ Lowering pass

Render IR (LIR - Low-Level IR)
├── Platform-specific widget selection
├── Resolved property values (converted to platform types)
├── Platform hints pre-populated
├── Responsive variants resolved
└── Example: FlexRow → Flutter: Row + MainAxisAlignment + CrossAxisAlignment

     ↓ Code generation

Platform Code
```

### Advantages of the split

1. **Clear phase boundaries** — Each IR level has a well-defined purpose and transformation
2. **Shared optimization** — Platform-neutral passes (text merging, dead code elimination) operate on HIR/MIR
3. **No semantic leakage** — Platform concerns stay in LIR
4. **Easier to add backends** — New backends implement LIR lowering; HIR/MIR are shared
5. **Better testability** — Each lowering pass is independently testable
6. **Parallel construction** — Semantic and layout inference can run concurrently

### Current IR issues

| Issue | Severity | Description |
|-------|----------|-------------|
| Responsive inferrer is a stub | Critical | Never produces ResponsiveVariant; no breakpoint detection |
| TargetIr not populated | High | Flutter/Compose/SwiftUI variants are defined but never constructed in IR crate |
| `_diagnostics` parameter ignored | Medium | IrBuilder accepts DiagnosticBag but never writes to it |
| `LayoutStrategy` duplicate enum | Low | Dead code — `LayoutStrategy` in layout.rs duplicates `LayoutIr` but is never used |
| `LayoutConstraints` unused | Low | Struct exists in ast-ir but never instantiated outside tests |
| `LayoutIr::ZStack` never emitted | Low | Defined in enum but `LayoutInferrer` never produces it |
| `LayoutIr::LazyList` never emitted | Low | Defined in enum but never produced |
| `aria-labelledby` not resolved | Medium | Stored as string; no cross-reference resolution |
| Text direction not inferred | Low | No `dir="auto"` handling |
| Heading level not validated | Low | `u32` used but HTML only defines h1-h6 |

### Missing IR features for 1.0

| Feature | Priority | Description |
|---------|----------|-------------|
| Responsive variants from media queries | Critical | Extract breakpoints from @media rules, produce ResponsiveVariant |
| TargetIr population | High | Populate Flutter/Compose/SwiftUI variants with widget names and properties |
| Flexible child semantics | High | `flex-grow`, `flex-shrink`, `flex-basis` per child (not just container) |
| Grid track structure | Medium | Parsed grid-template-columns/rows with track sizes, line names, areas |
| Absolute positioning data | Medium | top/right/bottom/left offsets per node |
| Event handler data | Medium | From JS frontend: click → onPressed, etc. |
| z-index ordering | Medium | Stack ordering information |
| Overflow behavior | Medium | Scroll vs clip vs visible |
| Accessibility resolution | Medium | aria-labelledby cross-reference resolution |
| Multi-platform hints | Low | TargetHints for different platforms on the same IR |
| Animation/transition data | Low | @keyframes → platform animation hints |
