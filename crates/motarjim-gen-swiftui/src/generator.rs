use crate::*;

/// Generates `SwiftUI` code from an IR tree.
#[derive(Debug, Clone)]
pub struct SwiftUIGenerator;

#[allow(clippy::unused_self)]
impl SwiftUIGenerator {
    /// Creates a new `SwiftUIGenerator`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generates Swift code from the given IR tree.
    #[must_use]
    pub fn generate(&self, tree: &IrTree) -> String {
        let mut w = CodeWriter::new(4);
        w.write_line("import SwiftUI");
        w.blank_line();

        if let Some(root) = tree.nodes.iter().find(|n| n.id == tree.root_id) {
            self.emit_root(tree, root, &mut w);
        }

        w.into_string()
    }

    /// Emits the root struct conforming to View.
    fn emit_root(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("struct GeneratedPage: View {");
        w.indent();
        w.write_line("var body: some View {");
        w.indent();
        if node.children.is_empty() {
            w.write_line("Color.clear");
        } else if node.children.len() == 1 {
            self.emit_children(tree, &node.children, w);
        } else {
            w.write_line("VStack {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        }
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("}");
    }

    /// Dispatches to the appropriate emitter based on semantic role.
    fn emit_widget(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        match &node.semantic {
            SemanticIr::Root => self.emit_root(tree, node, w),
            SemanticIr::Container
            | SemanticIr::Section
            | SemanticIr::Article
            | SemanticIr::Main
            | SemanticIr::Footer
            | SemanticIr::Header
            | SemanticIr::Aside => self.emit_container(tree, node, w),
            SemanticIr::Navigation => self.emit_navigation(tree, node, w),
            SemanticIr::NavigationBar => self.emit_nav_bar(tree, node, w),
            SemanticIr::HeroSection => self.emit_hero(tree, node, w),
            SemanticIr::Card => self.emit_card(tree, node, w),
            SemanticIr::Button => self.emit_button(tree, node, w),
            SemanticIr::Text | SemanticIr::Paragraph => self.emit_text(tree, node, w),
            SemanticIr::Heading { level } => self.emit_heading(tree, node, *level, w),
            SemanticIr::Image => self.emit_image(tree, node, w),
            SemanticIr::Icon => self.emit_icon(tree, node, w),
            SemanticIr::Input | SemanticIr::TextArea => self.emit_text_field(tree, node, w),
            SemanticIr::Select => self.emit_picker(tree, node, w),
            SemanticIr::Checkbox => self.emit_toggle(tree, node, w),
            SemanticIr::Radio => self.emit_picker(tree, node, w),
            SemanticIr::Form => self.emit_form(tree, node, w),
            SemanticIr::List | SemanticIr::LazyList => self.emit_list(tree, node, w),
            SemanticIr::ListItem => self.emit_list_item(tree, node, w),
            SemanticIr::Table => self.emit_table(tree, node, w),
            SemanticIr::TableRow | SemanticIr::TableCell => {
                self.emit_children(tree, &node.children, w);
            }
            SemanticIr::Row => self.emit_hstack(tree, node, w),
            SemanticIr::Column => self.emit_vstack(tree, node, w),
            SemanticIr::Stack => self.emit_zstack(tree, node, w),
            SemanticIr::Scroll => self.emit_scroll_view(tree, node, w),
            SemanticIr::Grid => self.emit_grid(tree, node, w),
            SemanticIr::Flex => self.emit_flex(tree, node, w),
            SemanticIr::Divider => w.write_line("Divider()"),
            SemanticIr::Spacer => w.write_line("Spacer()"),
            SemanticIr::Dialog => self.emit_dialog(tree, node, w),
            SemanticIr::Tooltip => self.emit_tooltip(tree, node, w),
            SemanticIr::Badge => self.emit_badge(tree, node, w),
            SemanticIr::IconButton => self.emit_icon_button(tree, node, w),
            SemanticIr::Chip => w.write_line("Label(\"chip\", systemImage: \"tag\")"),
            SemanticIr::Avatar => self.emit_avatar(tree, node, w),
            SemanticIr::Progress => w.write_line("ProgressView()"),
            SemanticIr::Skeleton => {
                w.write_line("Color.gray.opacity(0.3).frame(height: 20).cornerRadius(4)");
            }
            SemanticIr::Custom(name) => {
                w.write_line(&format!("// Custom view: {name}"));
            }
        }
    }

    /// Emits a container view, choosing VStack/HStack based on layout.
    fn emit_container(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        let view = match node.layout {
            LayoutIr::FlexRow => "HStack",
            LayoutIr::FlexColumn => "VStack",
            _ => "VStack",
        };
        w.write_line(&format!("{view} {{"));
        w.indent();
        // Apply padding modifier
        let padding_val = self.format_padding_modifier(&node.computed_style.padding);
        if !padding_val.is_empty() {
            w.write_line(&format!(".padding({padding_val})"));
        }
        if let Some(ref color) = node.computed_style.color {
            w.write_line(&format!(".background({})", self.format_color_swift(color)));
        }
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a `NavigationStack` wrapper.
    fn emit_navigation(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("NavigationStack {");
        w.indent();
        let non_bar_ids: Vec<NodeId> = node
            .children
            .iter()
            .filter(|cid| {
                !tree
                    .nodes
                    .iter()
                    .any(|n| n.id == **cid && n.semantic == SemanticIr::NavigationBar)
            })
            .copied()
            .collect();
        if !non_bar_ids.is_empty() {
            w.write_line("List {");
            w.indent();
            self.emit_children_for_ids(tree, &non_bar_ids, w);
            w.dedent();
            w.write_line("}");
        }
        if node.children.iter().any(|cid| {
            tree.nodes
                .iter()
                .any(|n| n.id == *cid && n.semantic == SemanticIr::NavigationBar)
        }) {
            w.write_line(".navigationTitle(\"Navigation\")");
        }
        w.dedent();
        w.write_line("}");
    }

    /// Emits a navigation bar modifier.
    fn emit_nav_bar(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line(".navigationTitle(\"Page\")");
        w.write_line(".navigationBarTitleDisplayMode(.inline)");
    }

    /// Emits a hero section.
    fn emit_hero(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("VStack {");
        w.indent();
        if let Some(ref color) = node.computed_style.color {
            w.write_line(&format!(".background({})", self.format_color_swift(color)));
        }
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
        w.write_line(".frame(maxWidth: .infinity)");
        w.write_line(".padding(24)");
    }

    /// Emits a Card view.
    fn emit_card(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("VStack {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
        w.write_line(".padding()");
        w.write_line(".background(Color(.systemBackground))");
        w.write_line(".cornerRadius(12)");
        w.write_line(".shadow(radius: 4)");
    }

    /// Emits a Button view.
    fn emit_button(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        if node.children.is_empty() {
            w.write_line("Button(\"Button\") { }");
        } else {
            // Collect text for button label
            let label_text = node.children.iter().find_map(|cid| {
                tree.nodes
                    .iter()
                    .find(|n| n.id == *cid && matches!(n.semantic, SemanticIr::Text))
            });
            if let Some(text_node) = label_text {
                // Use text content literally for simple buttons
                let _ = text_node;
                w.write_line("Button(action: { }) {");
                w.indent();
                self.emit_children(tree, &node.children, w);
                w.dedent();
                w.write_line("}");
            } else {
                w.write_line("Button(action: { }) {");
                w.indent();
                self.emit_children(tree, &node.children, w);
                w.dedent();
                w.write_line("}");
            }
        }
    }

    /// Emits a Text view.
    fn emit_text(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Text(\"Text content\")");
    }

    /// Emits a heading as styled Text.
    fn emit_heading(&self, _tree: &IrTree, _node: &IrNode, level: u32, w: &mut CodeWriter) {
        let font = match level {
            1 => "largeTitle",
            2 => "title",
            3 => "title2",
            4 => "title3",
            5 => "headline",
            _ => "body",
        };
        w.write_line(&format!("Text(\"Heading {level}\").font(.{font}).bold()"));
    }

    /// Emits an Image view.
    fn emit_image(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("AsyncImage(url: URL(string: \"https://example.com/image.png\")) {");
        w.indent();
        w.write_line("phase in");
        w.indent();
        w.write_line("switch phase {");
        w.indent();
        w.write_line("case .success(let image):");
        w.indent();
        w.write_line("image.resizable().scaledToFill()");
        w.dedent();
        w.write_line("case .failure(let error):");
        w.indent();
        let _ = w; // avoid unused warning - already used for Text
        w.write_line("Text(error.localizedDescription)");
        w.dedent();
        w.write_line("case .empty:");
        w.indent();
        w.write_line("ProgressView()");
        w.dedent();
        w.write_line("@unknown default:");
        w.indent();
        w.write_line("EmptyView()");
        w.dedent();
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.dedent();
        w.write_line("}");
    }

    /// Emits an icon using SF Symbols.
    fn emit_icon(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Image(systemName: \"star.fill\")");
    }

    /// Emits a `TextField`.
    fn emit_text_field(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("TextField(\"Input\", text: .constant(\"\"))");
        w.write_line(".textFieldStyle(.roundedBorder)");
    }

    /// Emits a Picker (for select/radio).
    fn emit_picker(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Picker(\"Option\", selection: .constant(0)) {");
        w.indent();
        w.write_line("Text(\"Option 1\").tag(0)");
        w.write_line("Text(\"Option 2\").tag(1)");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a Toggle (checkbox equivalent).
    fn emit_toggle(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Toggle(\"Checkbox\", isOn: .constant(false))");
    }

    /// Emits a Form.
    fn emit_form(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Form {");
        w.indent();
        w.write_line("Section {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a List.
    fn emit_list(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        if node.children.is_empty() {
            w.write_line("List {");
            w.indent();
            w.write_line("Text(\"No items\")");
            w.dedent();
            w.write_line("}");
        } else {
            w.write_line("List {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        }
    }

    /// Emits a list item.
    fn emit_list_item(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("HStack {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a table (unsupported in `SwiftUI`, use Grid instead).
    fn emit_table(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("// Table view - use List or Grid for tabular data");
    }

    /// Emits an `HStack`.
    fn emit_hstack(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("HStack {");
        w.indent();
        let style = &node.computed_style;
        if let Some(ref ai) = style.align_items {
            let value = match ai {
                motarjim_ast_html::AlignItems::FlexStart => ".top",
                motarjim_ast_html::AlignItems::FlexEnd => ".bottom",
                motarjim_ast_html::AlignItems::Center => ".center",
                _ => ".top",
            };
            w.write_line(&format!(".alignmentGuide(.top) {{ _ in {value} }}"));
        }
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a `VStack`.
    fn emit_vstack(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("VStack {");
        w.indent();
        let style = &node.computed_style;
        if let Some(ref ai) = style.align_items {
            let value = match ai {
                motarjim_ast_html::AlignItems::FlexStart => ".leading",
                motarjim_ast_html::AlignItems::FlexEnd => ".trailing",
                motarjim_ast_html::AlignItems::Center => ".center",
                _ => ".leading",
            };
            w.write_line(&format!(".frame(maxWidth: .infinity, alignment: {value})"));
        }
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a `ZStack`.
    fn emit_zstack(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("ZStack {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a `ScrollView`.
    fn emit_scroll_view(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("ScrollView {");
        w.indent();
        w.write_line("VStack {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a `LazyVGrid`.
    fn emit_grid(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("LazyVGrid(columns: [GridItem(.adaptive(minimum: 100))]) {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a Flex container as `HStack` or `VStack` based on layout.
    fn emit_flex(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        if node.layout == LayoutIr::FlexColumn {
            w.write_line("VStack {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        } else {
            w.write_line("HStack {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        }
    }

    /// Emits an alert dialog.
    fn emit_dialog(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("// Alert dialog");
        w.write_line(".alert(\"Dialog\", isPresented: .constant(true)) {");
        w.indent();
        w.write_line("Button(\"OK\") { }");
        w.dedent();
        w.write_line("} message: {");
        w.indent();
        w.write_line("Text(\"Dialog content\")");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a tooltip/popover.
    fn emit_tooltip(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("// Tooltip - use .popover or .help modifier");
        w.write_line("Text(\"Info\").help(\"Tooltip text\")");
    }

    /// Emits a badge.
    fn emit_badge(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Image(systemName: \"bell.fill\")");
        w.write_line(".badge(1)");
    }

    /// Emits an icon button.
    fn emit_icon_button(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Button(action: { }) {");
        w.indent();
        w.write_line("Image(systemName: \"star.fill\")");
        w.dedent();
        w.write_line("}");
    }

    /// Emits an avatar.
    fn emit_avatar(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Circle()");
        w.write_line(".fill(Color.gray)");
        w.write_line(".frame(width: 48, height: 48)");
    }

    /// Emits all children of a node.
    fn emit_children(&self, tree: &IrTree, children: &[NodeId], w: &mut CodeWriter) {
        for child_id in children {
            if let Some(child) = tree.nodes.iter().find(|n| n.id == *child_id) {
                self.emit_widget(tree, child, w);
            }
        }
    }

    /// Emits specific child IDs.
    fn emit_children_for_ids(&self, tree: &IrTree, ids: &[NodeId], w: &mut CodeWriter) {
        for child_id in ids {
            if let Some(child) = tree.nodes.iter().find(|n| n.id == *child_id) {
                self.emit_widget(tree, child, w);
            }
        }
    }

    /// Formats padding as a `SwiftUI` padding modifier string.
    fn format_padding_modifier(&self, ev: &EdgeValues) -> String {
        let top = ev.top;
        let right = ev.right;
        let bottom = ev.bottom;
        let left = ev.left;

        if (top - 0.0).abs() < f64::EPSILON
            && (right - 0.0).abs() < f64::EPSILON
            && (bottom - 0.0).abs() < f64::EPSILON
            && (left - 0.0).abs() < f64::EPSILON
        {
            String::new()
        } else if (top - right).abs() < f64::EPSILON
            && (top - bottom).abs() < f64::EPSILON
            && (top - left).abs() < f64::EPSILON
        {
            format!("{top}")
        } else if (top - bottom).abs() < f64::EPSILON && (right - left).abs() < f64::EPSILON {
            format!(".horizontal({right}).vertical({top})")
        } else {
            format!("EdgeInsets(top: {top}, leading: {left}, bottom: {bottom}, trailing: {right})")
        }
    }

    /// Converts a CSS color string to a `SwiftUI` Color expression.
    fn format_color_swift(&self, color: &str) -> String {
        let hex = color.trim_start_matches('#');
        let padded = if hex.len() <= 6 {
            format!("#{hex:0>6}")
        } else {
            format!("#{hex}")
        };
        format!("Color({padded:?})")
    }
}

impl Default for SwiftUIGenerator {
    fn default() -> Self {
        Self::new()
    }
}
