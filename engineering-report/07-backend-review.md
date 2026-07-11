# Backend Review

## Overview

Three code generators produce platform-native source code from the optimized `IrTree`. All three follow the same architecture: a recursive dispatch on `SemanticIr` variants that emits platform-specific code via the shared `CodeWriter`.

---

## Flutter (Dart) Generator

**Crate:** `motarjim-gen-flutter` | **LOC:** 1,033 | **Generator struct:** `FlutterGenerator`

### Generated Code Quality: 5/10

- Produces **valid Dart** code
- Output is a single `StatelessWidget` with a `Scaffold`
- Uses Material Design widgets exclusively
- **No theming, no state management, no routing**

### Architecture: 7/10

- Clean `emit_widget()` dispatch on `SemanticIr` (34/41 variants handled)
- `emit_children()` / `emit_children_for_ids()` for tree walking
- `format_edge_values_dart()` and `format_color_dart()` for CSS property formatting
- Parallel structure to Compose and SwiftUI generators (easy to maintain)

### Widget Mapping

| SemanticIr | Generated Widget |
|------------|-----------------|
| Root | `GeneratedPage` (StatelessWidget) |
| Container/Section/Article/etc. | `Row`, `Column`, or `Container` based on LayoutIr |
| Navigation | `Scaffold` + `AppBar` |
| Card | `Card` |
| Button | `ElevatedButton` |
| Text/Paragraph | `Text` |
| Heading { level } | `Text` with `TextStyle(fontSize, fontWeight)` |
| Image | `Image.network(...)` (hardcoded URL) |
| Input/TextArea | `TextField` with `OutlineInputBorder` |
| Select | `DropdownButton<String>` |
| Table | `Table` with `TableBorder.all()` |
| List | `ListView` |
| Grid | `GridView` with `SliverGridDelegateWithFixedCrossAxisCount(2)` |
| Dialog | `AlertDialog` |
| Progress | `LinearProgressIndicator()` |
| Skeleton | `SizedBox(width: infinity, height: 20)` |

### CSS Property Mapping: 3/10

- padding (via EdgeInsets)
- margin (via EdgeInsets)
- color (CSS hex → `Color(0xFF...)`)
- justify-content → MainAxisAlignment
- align-items → CrossAxisAlignment
- **All other CSS properties not mapped** (width, height, background, border, border-radius, box-shadow, font-family, font-size, text-align, gap, flex-grow, opacity, overflow, position, z-index)

### Known Bugs

| Bug | Location | Description |
|-----|----------|-------------|
| `emit_table_cell` wrong widget | generator.rs:467 | Writes `TableRow(` instead of table cell widget — copy-paste error |

### Testing: 14 tests

- No snapshot tests; all use substring matching
- Missing tests for: tables, forms, navigation, dialogs, grids, CSS property output

---

## Jetpack Compose (Kotlin) Generator

**Crate:** `motarjim-gen-compose` | **LOC:** 904 | **Generator struct:** `ComposeGenerator`

### Generated Code Quality: 5/10

- Produces **valid Kotlin**
- Uses Material3 (`material3.*`) exclusively
- Single `@Composable` function
- **Best CSS property mapping of the three generators**

### Architecture: 7/10

- Same dispatch pattern as Flutter
- `build_modifier()` abstraction chains Compose `Modifier` calls
- Margin treated as additional padding (correct for Compose)

### Widget Mapping

| SemanticIr | Generated Composable |
|------------|---------------------|
| Root | `@Composable fun GeneratedPage()` with `Column(Modifier.fillMaxSize())` |
| Container/Section/etc. | `Box`/`Row`/`Column` based on LayoutIr |
| Navigation | `Scaffold` + `TopAppBar` |
| Card | `Card` |
| Button | `Button(onClick = { })` |
| Image | `Image(painter = rememberImagePainter(...))` (requires coil) |
| Input/TextArea | `OutlinedTextField` |
| Select | `DropdownMenu` |
| Table | **Comment only** — "Table layout not directly available in Compose" |
| Grid | `LazyVerticalGrid(columns = GridCells.Fixed(2))` |
| Dialog | `AlertDialog` |
| Tooltip | **Stub** — hardcoded `Box { Text("Hover me") }` |

### CSS Property Mapping: 4/10

- padding (via `Modifier.padding(...)`)
- margin (via `Modifier.padding(...)` — mapped as additional padding)
- color (CSS hex → `Color(0xFF...)`)
- **width** (parsed from px string → `Modifier.width(N.dp)`)
- **height** (parsed from px string → `Modifier.height(N.dp)`)
- justify-content → Arrangement
- align-items → Alignment

### Known Issues

| Issue | Location | Description |
|-------|----------|-------------|
| LazyColumn uses fake data | generator.rs | `listOf(1)` instead of actual children |
| Grid uses fake data | generator.rs | `listOf(1)` instead of actual children |
| Image requires `coil` | generator.rs | Uses `rememberImagePainter` which is not in standard Compose |
| No coil import added | generator.rs | Missing dependency |
| Tooltip is a stub | generator.rs | Hardcoded placeholder content |

### Testing: 12 tests

- No snapshot tests; all use substring matching
- Missing tests for: tables, forms, navigation, dialogs, grids, CSS property output

---

## SwiftUI (Swift) Generator

**Crate:** `motarjim-gen-swiftui` | **LOC:** 783 | **Generator struct:** `SwiftUIGenerator`

### Generated Code Quality: 3/10

- **Several bugs produce invalid Swift**
- Uses modern APIs (NavigationStack, AsyncImage, LazyVGrid)
- Uses iOS 15+ APIs without availability annotations

### Architecture: 7/10

- Same dispatch pattern as Flutter and Compose
- SwiftUI modifier chain pattern (`.padding()`, `.background()`, etc.)

### Widget Mapping

| SemanticIr | Generated View |
|------------|----------------|
| Root | `struct GeneratedPage: View` with `VStack` or `Color.clear` |
| Container/Section/etc. | `HStack`/`VStack` based on LayoutIr |
| Navigation | `NavigationStack { List { ... } .navigationTitle(...) }` |
| Card | `VStack` with `.padding()`, `.background()`, `.cornerRadius()`, `.shadow()` |
| Button | `Button("Button") { }` |
| Image | `AsyncImage` with loading/success/failure states (**best of all 3**) |
| Input/TextArea | `TextField("Input", text: .constant(""))` + `.textFieldStyle(.roundedBorder)` |
| Select | `Picker("Option", selection: .constant(0))` with hardcoded options |
| Form | `Form { Section { ... } }` |
| Table | **Comment only** — "use List or Grid for tabular data" |
| Grid | `LazyVGrid(columns: [GridItem(.adaptive(minimum: 100))])` |
| Dialog | `.alert("Dialog", isPresented: .constant(true))` (modifier) |

### CSS Property Mapping: 2/10

- padding (via `.padding(EdgeInsets(...))`)
- color (CSS hex → `Color("#RRGGBB")`)
- **All other CSS properties not mapped**

### Critical Bugs

| Bug | Location | Description |
|-----|----------|-------------|
| `emit_hstack` alignment syntax wrong | generator.rs:379 | `.alignmentGuide(.top) { _ in .top }` — `.top` is not a valid value; should be CGFloat |
| `emit_dialog` orphan modifier | generator.rs:456-464 | `.alert(...)` emitted as standalone line without parent view |
| `emit_nav_bar` modifier chain invalid | generator.rs:163-165 | Modifiers emitted as standalone lines from child handler; should be applied by parent |
| `justify_content` not mapped | — | Flutter and Compose have this; SwiftUI does not |
| width/height not mapped | — | Present in Compose generator but not SwiftUI |

### Testing: 11 tests

- No snapshot tests; all use substring matching
- Missing tests for: tables, forms, navigation, scroll, grids, CSS property output

---

## Cross-Cutting Issues

### Shared Problems (All 3 Generators)

| Issue | Severity | Description |
|-------|----------|-------------|
| CSS properties mostly unmapped | Critical | Only 2-6 of 50+ CSS properties produce platform code |
| Hardcoded image URLs | High | All generators use `"https://example.com/image.png"` |
| Hardcoded icon names | High | All use `star`/`star.fill` — no data from IR |
| Hardcoded form data | High | No placeholder, label, validation wired from Node attributes |
| No accessibility output | High | AccessibilityInfo not used by any generator |
| No event handler wiring | High | JS `find_dom_event_bindings()` output not consumed |
| No source maps | High | No mapping from generated code back to source |
| `#[allow(clippy::unused_self)]` | Low | Many methods could be free functions |
| Formatter platform modules unused | Low | `dart::write_class`, `kotlin::write_fun`, `swift::write_struct` exist but generators hand-code output |

### Comparison Table

| Dimension | Flutter | Compose | SwiftUI |
|-----------|:-------:|:-------:|:-------:|
| Overall Score | 5/10 | 5/10 | 4/10 |
| Generated Code Quality | 5/10 | 5/10 | 3/10 |
| Architecture | 7/10 | 7/10 | 7/10 |
| Readability | 7/10 | 7/10 | 7/10 |
| CSS Property Mapping | 3/10 | 4/10 | 2/10 |
| Widget Coverage | 34/41 | 34/41 | 34/41 |
| Tests | 14 | 12 | 11 |
| Critical Bugs | 1 | 0 | 3 |
| Missing CSS Mapping | ~45 props | ~44 props | ~48 props |

### What needs to happen for 1.0

1. **Fix all known bugs** (SwiftUI alignment, dialog, navbar; Flutter table cell)
2. **Map all CSS properties** that have computed values (minimum: width, height, background, border, border-radius, font-size, font-family, text-align, gap, opacity, overflow, position, box-shadow, flex-grow/shrink)
3. **Wire data from IR** (image `src`, icon name, form field labels/placeholders)
4. **Wire event bindings** from JS frontend
5. **Add accessibility attributes** to generated code
6. **Add snapshot/golden tests** for all examples
7. **Use formatter platform modules** instead of hand-coded output
8. **Support multi-platform output** from single pass (remove `TargetIr::Generic` usage)
