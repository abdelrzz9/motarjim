# Motarjim Architecture v2

## Design Philosophy

Motarjim is a production compiler platform that compiles HTML/CSS into multiple native UI frameworks (Flutter, SwiftUI, Jetpack Compose). It is engineered as a long-lived compiler infrastructure with the same architectural rigor as LLVM, rustc, or Swift's compiler — while remaining appropriately scoped for a UI compiler.

## Final Crate Structure

```
crates/
├── frontend/
│   ├── motarjim-lexer/          # HTML + CSS tokenizer
│   ├── motarjim-parser/         # HTML + CSS recursive descent parser
│   └── motarjim-js/             # JavaScript frontend (event extraction)
│
├── ast/
│   ├── motarjim-ast-html/       # HTML AST types (Document, Element, Node)
│   ├── motarjim-ast-css/        # CSS AST types (Stylesheet, Rule, Declaration)
│   └── motarjim-ast-js/         # JS AST types
│
├── style/
│   ├── motarjim-selectors/      # CSS selector parsing, matching, specificity
│   └── motarjim-css/            # Cascade resolution, computed values → StyleProperties
│
├── layout/
│   └── motarjim-layout/         # Flexbox, Grid, constraints, intrinsic sizing,
│                                #   baselines, wrapping, overflow, alignment
│                                #   Input: StyledDocument → Output: LayoutTree
│
├── ir/
│   └── motarjim-ir/             # ALL IR definitions in one crate:
│       ├── hir/                 #   HIR types (SemanticRole, LayoutIntent, StyleProperties)
│       ├── mir/                 #   MIR types (optimized, validated HIR)
│       ├── lir/                 #   LIR types (RenderNode primitives)
│       ├── visitor/             #   Visitor, VisitorMut, Fold, Transform traits
│       ├── validate/            #   IR validation rules
│       └── arena/               #   Arena-backed storage, stable NodeId
│
├── lowering/
│   └── motarjim-lowering/       # HIR → MIR → LIR transforms
│                                #   Responsive resolution, layout assignment,
│                                #   style flattening, rendering primitive lowering
│
├── pass-manager/
│   └── motarjim-pass-manager/   # Pass trait, PassManager, scheduling, analysis results
│
├── optimize/
│   ├── motarjim-opt/            # IR optimization passes (text merge, DCE, flatten, etc.)
│   └── motarjim-style-opt/      # Style normalization/deduplication
│
├── query/
│   └── motarjim-query/          # Query engine (Salsa-like): cached, dependency-tracked
│                                #   queries with fine-grained invalidation
│
├── backend/
│   ├── motarjim-backend/        # Backend trait, LIR consumer API, BackendRegistry
│   ├── motarjim-gen-flutter/    # LIR → Dart/Flutter
│   ├── motarjim-gen-compose/    # LIR → Kotlin/Jetpack Compose
│   └── motarjim-gen-swiftui/    # LIR → Swift/SwiftUI
│
├── infra/
│   ├── motarjim-span/           # SourceSpan, SourceLocation (lightweight)
│   ├── motarjim-source/         # SourceFile, SourceMap, LineIndex, UTF-8/16 offsets
│   ├── motarjim-diag/           # Diagnostic, Severity, ErrorCode, emitter
│   ├── motarjim-fs/             # Abstract filesystem (real, virtual, remote)
│   ├── motarjim-config/         # motarjim.json/.toml loading
│   ├── motarjim-profiling/      # Phase timing, metrics
│   ├── motarjim-formatter/      # CodeWriter for platform code output
│   ├── motarjim-cache/          # Content-addressable artifact cache
│   └── motarjim-serialize/      # Serde helpers for AST/IR
│
├── session/
│   └── motarjim-session/        # Session: single object passed everywhere
│                                #   Contains: Config, Diagnostics, SourceMap,
│                                #   Cache, Filesystem, Target, Profiling, ...
│
├── driver/
│   └── motarjim-driver/         # Compiler session lifecycle, pipeline orchestration
│                                #   (replaces motarjim-core)
│
├── plugin/
│   └── motarjim-plugin/         # Plugin trait, PluginRegistry, dynamic loading
│                                #   Supports: backends, passes, lints, analysis, diagnostics
│
├── consumers/
│   ├── motarjim-cli/            # CLI binary (compile, watch, init, check)
│   ├── motarjim-lsp/            # Language server
│   ├── motarjim-wasm/           # WASM bindings
│   └── motarjim-ffi/            # C FFI bindings
│
└── tooling/
    ├── xtask/                   # Build scripts, codegen, diagrams
    └── motarjim-test-utils/     # Shared test infrastructure
```

## Dependency Graph

```
                    ┌──────────────────────────────────────────┐
                    │            CONSUMERS                     │
                    │  cli, lsp, wasm, ffi                     │
                    └────────────┬─────────────────────────────┘
                                 │
                    ┌────────────▼─────────────────────────────┐
                    │            DRIVER                        │
                    │  motarjim-driver                         │
                    │  (orchestrates pipeline)                 │
                    └────────────┬─────────────────────────────┘
                                 │
              ┌──────────────────┼────────────────────┬──────────────────┐
              │                  │                    │                  │
     ┌────────▼───────┐  ┌──────▼───────┐   ┌───────▼────────┐  ┌─────▼──────────┐
     │   SESSION      │  │   PLUGIN     │   │    QUERY       │  │ INFRASTRUCTURE │
     │ motarjim-      │  │ motarjim-    │   │  motarjim-     │  │ span, source,  │
     │   session      │  │   plugin     │   │   query        │  │ diag, fs,      │
     └────────▲───────┘  └──────┬───────┘   └───────┬────────┘  │ config, cache,  │
              │                 │                   │           │ profiling,      │
              │         ┌───────▼───────┐           │           │ formatter,      │
              │         │   BACKEND     │           │           │ serialize       │
              │         │ motarjim-     │           │           └─────────────────┘
              │         │   backend     │           │
              │         │ gen-flutter   │           │
              │         │ gen-compose   │           │
              │         │ gen-swiftui   │           │
              │         └───────▲───────┘           │
              │                 │                   │
              │         ┌───────▼───────┐           │
              │         │  LOWERING     │           │
              │         │ motarjim-     │           │
              │         │   lowering    │           │
              │         └───────┬───────┘           │
              │                 │                   │
              │         ┌───────▼───────┐           │
              │         │    IR         │           │
              │         │ motarjim-ir   │           │
              │         │   (hir, mir,  │           │
              │         │    lir,       │           │
              │         │    visitor,   │           │
              │         │    validate,  │           │
              │         │    arena)     │           │
              │         └───────┬───────┘           │
              │                 │                   │
              │         ┌───────▼───────┐           │
              │         │   LAYOUT      │           │
              │         │ motarjim-     │           │
              │         │   layout      │           │
              │         └───────┬───────┘           │
              │                 │                   │
              │         ┌───────▼───────┐           │
              │         │   STYLE       │           │
              │         │ motarjim-css  │           │
              │         │ motarjim-     │           │
              │         │   selectors   │           │
              │         └───────┬───────┘           │
              │                 │                   │
              │         ┌───────▼───────┐           │
              │         │  FRONTEND     │           │
              │         │ motarjim-     │           │
              │         │   lexer       │           │
              │         │ motarjim-     │           │
              │         │   parser      │           │
              │         │ motarjim-js   │           │
              │         └───────┬───────┘           │
              │                 │                   │
              │         ┌───────▼───────┐           │
              └─────────┤     AST       │           │
                        │ motarjim-ast-*│           │
                        └───────────────┘           │
                                                    │
              ┌─────────────────────────────────────┘
              │
     ┌────────▼────────┐
     │ PASS MANAGER    │
     │ motarjim-pass-  │
     │   manager       │
     └────────┬────────┘
              │
     ┌────────▼────────┐
     │   OPTIMIZE      │
     │ motarjim-opt    │
     │ motarjim-       │
     │   style-opt     │
     └─────────────────┘
```

## Compiler Pipeline

```
Source (HTML + CSS)
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│                      FRONTEND                               │
│                                                             │
│  1.  Lex (motarjim-lexer)                                   │
│      Input:  Raw source text                                │
│      Output: Token<TokenKind> with SourceSpan               │
│                                                             │
│  2.  Parse (motarjim-parser)                                │
│      Input:  Token streams                                  │
│      Output: Document (HTML AST) + CssStylesheet            │
│      Error:  Recovery with multiple diagnostics             │
│                                                             │
│  3.  DOM Construction (motarjim-parser, implicit)           │
│      Input:  Document                                       │
│      Output: DomTree (flat node array with parent/child)    │
│      Note:   The Document IS the DOM by construction        │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                      MIDDLE-END                             │
│                                                             │
│  4.  Style Resolution (motarjim-css + motarjim-selectors)   │
│      Input:  DomTree + CssStylesheet                        │
│      Output: HashMap<NodeId, StyleProperties>               │
│      Note:   StyleProperties is CSS-agnostic:               │
│              EdgeInsets, ColorRgba, Length, FontWeight, etc.│
│              CSS concepts (display, flex-direction) are     │
│              resolved into LayoutIntent here.               │
│                                                             │
│  5.  Layout (motarjim-layout)                               │
│      Input:  DomTree + StyleProperties                      │
│      Output: LayoutTree                                     │
│      Process: Flexbox, Grid, block/inline layout,           │
│               constraints, intrinsic sizing, baselines,     │
│               wrapping, overflow, alignment, positioning    │
│                                                             │
│  6.  HIR Construction (motarjim-ir::hir)                    │
│      Input:  DomTree + LayoutTree + StyleProperties         │
│      Output: HiTree                                         │
│      Process: Semantic inference (tag→role),                │
│               layout inference (CSS→LayoutIntent),          │
│               accessibility inference,                      │
│               responsive variant extraction.                │
│               Platform-neutral. NO CSS concepts leak.       │
│                                                             │
│  7.  HIR Validation (motarjim-ir::validate)                 │
│      Input:  HiTree                                         │
│      Output: Validated<HiTree> or diagnostics               │
│      Checks: Tree integrity, no orphans, no cycles,         │
│              consistent roles, valid style values           │
│                                                             │
│  8.  MIR Lowering (motarjim-lowering)                       │
│      Input:  HiTree                                         │
│      Output: MiTree                                         │
│      Process: Layout fully resolved, responsive variants    │
│               resolved, explicit positioning,               │
│               style properties canonicalized                 │
│                                                             │
│  9.  Optimization (motarjim-opt + motarjim-pass-manager)    │
│      Input:  MiTree                                         │
│      Output: Optimized MiTree                               │
│      Passes: MergeText, CollapseWhitespace, DCE,            │
│              FlattenContainers, InlineConstants,            │
│              StyleDeduplication, LayoutSimplify,            │
│              [user-registered passes]                       │
│                                                             │
│  10. MIR Validation (motarjim-ir::validate)                 │
│      Input:  Optimized MiTree                               │
│      Output: Validated<MiTree> or diagnostics               │
│                                                             │
│  11. LIR Lowering (motarjim-lowering)                       │
│      Input:  Optimized MiTree                               │
│      Output: LiTree                                         │
│      Process: Map (semantic, layout, style) → RenderNode    │
│               RenderText, RenderBox, RenderFlex,            │
│               RenderGrid, RenderStack, RenderScroll,        │
│               RenderImage, RenderInput.                     │
│               NO platform-specific concepts.                │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                      BACKEND                                │
│                                                             │
│  12. Code Generation (motarjim-gen-*)                       │
│      Input:  LiTree                                         │
│      Output: Platform source code (Dart/Kotlin/Swift)       │
│      Process: Backend maps RenderNode variants to           │
│               platform widgets. Uses CodeWriter.            │
│               Backend NEVER sees HTML, CSS, HIR, or MIR.    │
│                                                             │
│  13. Output (motarjim-formatter)                            │
│      Input:  Raw generated code                             │
│      Output: Formatted platform source code                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Session Design

```rust
/// The central context object passed to every compilation phase.
/// Inspired by rustc's Session.
pub struct Session {
    /// Compiler configuration (motarjim.json).
    pub config: Arc<Config>,
    /// Output sink for diagnostics.
    pub diagnostics: Arc<DiagnosticSink>,
    /// Source file map (multi-file support).
    pub source_map: Arc<SourceMap>,
    /// Abstract file system.
    pub file_system: Arc<dyn FileSystem>,
    /// Compilation artifact cache.
    pub cache: Arc<ArtifactCache>,
    /// Profiling session.
    pub profiling: Arc<ProfilingSession>,
    /// Target platform information.
    pub target: TargetInfo,
}

impl Session {
    /// Emit a diagnostic.
    pub fn emit(&self, diagnostic: Diagnostic);
    /// Check if compilation was cancelled.
    pub fn is_cancelled(&self) -> bool;
}
```

Every phase receives `&Session`. This eliminates the need to thread Config, Diagnostics, Filesystem, etc. individually through every function call.

## IR Architecture (Single Crate)

```
motarjim-ir/
├── Cargo.toml                  # Depends only on smol_str, smallvec, slotmap
├── src/
│   ├── lib.rs                  # Re-exports all modules
│   ├── hir/
│   │   ├── mod.rs
│   │   ├── node.rs             # HiNode, HiTree
│   │   ├── semantic.rs         # SemanticRole enum
│   │   ├── layout.rs           # LayoutIntent enum
│   │   └── style.rs            # StyleProperties (CSS-agnostic)
│   ├── mir/
│   │   ├── mod.rs
│   │   └── node.rs             # MiNode (structurally identical to HiNode)
│   ├── lir/
│   │   ├── mod.rs
│   │   ├── node.rs             # RenderNode enum + variants
│   │   ├── primitives.rs       # Length, Color, EdgeInsets, etc.
│   │   └── tree.rs             # LiTree
│   ├── visitor/
│   │   ├── mod.rs
│   │   ├── walk.rs             # Default walk implementations
│   │   ├── visitor.rs          # Visitor trait (immutable reference)
│   │   ├── visitor_mut.rs      # VisitorMut trait (mutable reference)
│   │   ├── fold.rs             # Fold trait (transform → new tree)
│   │   └── transform.rs        # Transform trait (in-place rewrite)
│   ├── validate/
│   │   ├── mod.rs
│   │   ├── tree.rs             # TreeIntegrity, NoCycles, NoOrphans
│   │   └── style.rs            # StyleConsistency, ValidValues
│   └── arena/
│       ├── mod.rs
│       └── id.rs               # Stable NodeId (slotmap key)
```

### Visitor Traits

```rust
/// Immutable traversal. Analysis passes use this.
pub trait Visitor {
    fn visit_hi_node(&mut self, node: &HiNode);
    fn visit_hi_tree(&mut self, tree: &HiTree);
    fn visit_mi_node(&mut self, node: &MiNode);
    fn visit_mi_tree(&mut self, tree: &MiTree);
    fn visit_render_node(&mut self, node: &RenderNode, ctx: &VisitorContext);
    fn visit_li_tree(&mut self, tree: &LiTree);
}

/// Mutable traversal. Transform passes use this.
pub trait VisitorMut {
    fn visit_hi_node_mut(&mut self, node: &mut HiNode);
    fn visit_mi_node_mut(&mut self, node: &mut MiNode);
    fn visit_render_node_mut(&mut self, node: &mut RenderNode);
}

/// Recursive tree transformation returning a new tree.
pub trait Fold<T> {
    fn fold(&mut self, input: T) -> T;
}

/// In-place tree transformation.
pub trait Transform {
    fn transform(&mut self, tree: &mut MiTree) -> Result<(), Vec<Diagnostic>>;
}
```

### Stable Node IDs

```rust
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    /// Stable node identifier. Never changes during compilation.
    /// Not an index. Not reused after deletion.
    pub struct NodeId;
}

/// Arena-backed node storage.
pub struct NodeArena<T> {
    arena: SlotMap<NodeId, T>,
}

impl<T> NodeArena<T> {
    pub fn insert(&mut self, value: T) -> NodeId;
    pub fn get(&self, id: NodeId) -> Option<&T>;
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T>;
    pub fn remove(&mut self, id: NodeId) -> Option<T>;
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)>;
}
```

### HIR Types

```rust
// SemanticRole — platform-neutral, CSS-agnostic
pub enum SemanticRole {
    Root, Navigation, NavigationBar, HeroSection, Card,
    Button, Text, Heading(u32), Paragraph, Image, Icon,
    Input(InputType), TextArea, Select, Checkbox, Radio,
    Form, List, ListItem, Table, TableRow, TableCell,
    Section, Article, Aside, Footer, Header, Main,
    Dialog, Tooltip, Badge, Divider, Spacer,
    Container, Grid, Flex, Column, Row, Stack,
    Scroll, LazyList, IconButton, Chip, Avatar,
    Progress, Skeleton, Custom(SmolStr),
}

// LayoutIntent — platform-neutral, CSS-agnostic
pub enum LayoutIntent {
    FlexRow { wrap: FlexWrap, gap: Length },
    FlexColumn { wrap: FlexWrap, gap: Length },
    Grid { columns: Vec<Length>, rows: Vec<Length>, gap: Length },
    Stack,
    ZStack,
    Scroll(Axis),
    LazyList(Axis),
    Absolute, Relative, Static, Sticky, Fixed,
    Flow, Inline, InlineBlock, Table,
    None,
}

// StyleProperties — NO CSS types. All concepts normalized.
pub struct StyleProperties {
    pub sizing: Sizing,         // Width, Height, Min/Max, Overflow
    pub spacing: Spacing,       // Margin, Padding as EdgeInsets
    pub typography: Typography, // FontSize, FontWeight, LineHeight, etc.
    pub decoration: Decoration, // Background, Border, Shadow, Opacity
    pub positioning: Positioning, // Position, ZIndex, Transform
    pub effects: Effects,       // Transition, Cursor, PointerEvents
}

pub struct EdgeInsets {
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
}

pub enum Length {
    Px(f64),
    Percent(f64),
    Auto,
    Inherit,
    FitContent,
    MaxContent,
    MinContent,
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
```

### LIR Types

```rust
pub enum RenderNode {
    Box(RenderBox),
    Text(RenderText),
    Image(RenderImage),
    Flex(RenderFlex),
    Grid(RenderGrid),
    Stack(RenderStack),
    Scroll(RenderScroll),
    Input(RenderInput),
    Custom { role: SmolStr, props: Vec<(SmolStr, Value)> },
}

pub struct RenderBox {
    pub id: NodeId,
    pub sizing: Sizing,
    pub spacing: Spacing,
    pub decoration: Decoration,
    pub effects: Effects,
    pub children: Vec<RenderNode>,
}

pub struct RenderText {
    pub id: NodeId,
    pub content: String,
    pub typography: Typography,
    pub color: Color,
    pub align: TextAlign,
}

pub struct RenderFlex {
    pub id: NodeId,
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub gap: Length,
    pub justify: JustifyContent,
    pub align: AlignItems,
    pub children: Vec<RenderNode>,
}

pub struct liTree {
    pub root: RenderNode,
    pub metadata: GenerationMeta,
}
```

## Layout Crate

`motarjim-layout` is a dedicated subsystem that takes styled DOM nodes and produces a layout tree with resolved positions and sizes.

```rust
// motarjim-layout/src/lib.rs

pub struct LayoutEngine {
    flexbox: FlexboxEngine,
    grid: GridEngine,
    positioning: PositioningEngine,
}

impl LayoutEngine {
    pub fn layout(
        &self,
        dom: &DomTree,
        styles: &HashMap<NodeId, StyleProperties>,
    ) -> LayoutTree;
}

pub struct LayoutTree {
    pub nodes: SlotMap<NodeId, LayoutNode>,
    pub root_id: NodeId,
}

pub struct LayoutNode {
    pub id: NodeId,
    pub layout_intent: LayoutIntent,
    pub resolved_position: Rect,
    pub resolved_size: Size,
    pub intrinsic_size: Size,
    pub baseline: f64,
    pub overflow: OverflowState,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
}
```

The layout crate handles:
- **Flexbox**: Main/cross axis sizing, distribution, wrapping
- **Grid**: Track sizing, cell placement, gaps
- **Block/Inline flow**: Normal document flow
- **Positioning**: Absolute, relative, fixed, sticky
- **Constraints**: Min/max width/height, aspect ratio
- **Intrinsic sizing**: Min-content, max-content, fit-content
- **Baselines**: Text alignment across flex/grid items
- **Overflow**: Scroll, hidden, visible resolution

## Lowering Crate

`motarjim-lowering` is a dedicated crate for all HIR→MIR→LIR transforms. This is a pure transformation layer with no side effects.

```rust
// motarjim-lowering/src/lib.rs

pub struct LoweringPipeline {
    hir_to_mir: HirToMirLowering,
    mir_to_lir: MirToLirLowering,
}

impl LoweringPipeline {
    pub fn lower(&self, hir: HiTree, session: &Session) -> Result<(MiTree, LiTree), Vec<Diagnostic>>;
}

// HIR → MIR: Resolve variants, canonicalize layout, flatten style
pub struct HirToMirLowering;
impl HirToMirLowering {
    pub fn lower(&self, hir: &HiTree) -> Result<MiTree, Vec<Diagnostic>>;
}

// MIR → LIR: Map semantic+layout to rendering primitives
pub struct MirToLirLowering;
impl MirToLirLowering {
    pub fn lower(&self, mir: &MiTree) -> Result<LiTree, Vec<Diagnostic>>;
}
```

## Query Engine

Before implementing advanced incremental compilation, build a Salsa-like query engine.

```rust
pub trait Query {
    type Key: Clone + Eq + Hash + Send;
    type Value: Clone + Send;
    fn execute(&self, key: &Self::Key, db: &dyn QueryDb) -> Self::Value;
}

pub trait QueryDb {
    fn execute_query<Q: Query + 'static>(&self, query: &Q, key: &Q::Key) -> Q::Value;
}

pub struct QueryEngine {
    storage: DashMap<QueryKey, CachedResult>,
    dependencies: DependencyGraph,
}

impl QueryEngine {
    pub fn get_or_compute<Q: Query + 'static>(
        &self,
        query: &Q,
        key: &Q::Key,
    ) -> Q::Value;

    pub fn invalidate(&self, changed_files: &[FilePath]);
}

// Example queries:
impl Query for ParseHtmlQuery { /* FilePath → Document */ }
impl Query for ParseCssQuery  { /* FilePath → Stylesheet */ }
impl Query for ResolveStyles  { /* FilePath → StyleMap */ }
impl Query for BuildLayout    { /* FilePath → LayoutTree */ }
impl Query for BuildHir       { /* FilePath → HiTree */ }
impl Query for OptimizeMir    { /* (FilePath, PassSetHash) → MiTree */ }
impl Query for LowerToLir     { /* FilePath → LiTree */ }
impl Query for GenerateCode   { /* (FilePath, BackendId) → String */ }
```

## Driver

```rust
pub struct Driver {
    session: Arc<Session>,
    query_engine: Arc<QueryEngine>,
    pass_manager: Arc<PassManager>,
    backend_registry: BackendRegistry,
}

impl Driver {
    pub fn compile(
        &self,
        source: &str,
        target: &str,
        options: &CompileOptions,
    ) -> Result<CompileResult, Vec<Diagnostic>>;

    pub fn compile_file(
        &self,
        path: &Path,
        options: &CompileOptions,
    ) -> Result<CompileResult, Vec<Diagnostic>>;
}

// The compile method delegates to the query engine:
// 1. query_engine.get_or_compute(&ParseHtml, path)
// 2. query_engine.get_or_compute(&ParseCss, path)
// 3. query_engine.get_or_compute(&ResolveStyles, path)
// 4. query_engine.get_or_compute(&BuildLayout, path)
// 5. query_engine.get_or_compute(&BuildHir, path)
// 6. query_engine.get_or_compute(&OptimizeMir, path)
// 7. query_engine.get_or_compute(&LowerToLir, path)
// 8. query_engine.get_or_compute(&GenerateCode, (path, target))
```

## Migration Plan (Updated)

### Phase 1: IR unification + Session
- Merge `motarjim-ast-ir` types into `motarjim-ir` with modules
- Create `motarjim-session` crate
- Create `motarjim-source` crate (move SourceFile, add SourceMap)
- Create `motarjim-layout` crate (move layout logic from `motarjim-ir`)
- Rename `motarjim-core` → `motarjim-driver`
- All existing code continues to work with deprecated re-exports

### Phase 2: Style normalization
- Create CSS-agnostic `StyleProperties` in `motarjim-ir::hir`
- Add conversion from `ComputedStyle` → `StyleProperties`
- Keep `ComputedStyle` for backward compatibility in CSS engine
- The IR no longer knows CSS exists

### Phase 3: Arena + Node IDs
- Introduce `slotmap` for stable `NodeId`
- Replace `Vec<IrNode>` indexing with `SlotMap<NodeId, IrNode>`
- Keep old `Vec`-based API for backward compatibility with deprecation

### Phase 4: Visitors
- Add `visitor/` module to `motarjim-ir`
- Add `Visitor`, `VisitorMut`, `Fold`, `Transform` traits
- Implement default walking logic
- Rewrite optimization passes to use visitors

### Phase 5: Lowering crate
- Create `motarjim-lowering`
- Move HIR→MIR transforms from `motarjim-ir` builder
- Implement MIR→LIR lowering
- Feature-gate old IR builder path

### Phase 6: Layout crate
- Make `motarjim-layout` a real standalone engine
- Move flexbox/grid/positioning logic from `motarjim-ir` layout inference
- Layout becomes an explicit pipeline stage instead of a sub-phase of HIR building

### Phase 7: Query engine
- Create `motarjim-query` crate
- Implement Salsa-like query caching
- Wire pass manager and lowering through queries
- Incremental compilation becomes a property of the query engine

### Phase 8: Plugin system expansion
- Add `DiagnosticProducer`, `LintRule` support
- Dynamic library loading
- Example plugins

### Phase 9: Legacy removal
- Remove deprecated APIs
- Remove `ComputedStyle` from IR/optimizer/generators
- Remove old `IrNode`/`IrTree` types
- All crates use new architecture exclusively

## Key Principles

1. **IR never knows about CSS or platforms** — `StyleProperties`, `SemanticRole`, `LayoutIntent`, and `RenderNode` are completely backend-agnostic.
2. **Session is the only context** — Every phase receives `&Session`. No threading of individual config/cache/diag references.
3. **One IR crate** — All IR types, visitors, validation, and arena live in `motarjim-ir`. `motarjim-lowering` performs transforms between levels.
4. **Stable NodeIds** — slotmap-backed. Never reuse indices. Never invalidate on deletion.
5. **Layout is its own subsystem** — Flexbox, Grid, constraints, intrinsic sizing are not CSS concerns. They are layout concerns.
6. **Source mapping is explicit** — `motarjim-source` owns `SourceFile`, `SourceMap`, line/column/offset conversions for LSP.
7. **Query engine before incremental** — Build Salsa-like caching first. Incremental compilation emerges naturally.
8. **Driver not core** — The word "core" is meaningless. "Driver" communicates session lifecycle and pipeline orchestration.
