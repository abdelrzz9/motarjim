use crate::*;

/// Generates Jetpack Compose/Kotlin code from an IR tree.
#[derive(Debug, Clone)]
pub struct ComposeGenerator;

#[allow(clippy::unused_self)]
impl ComposeGenerator {
    /// Creates a new `ComposeGenerator`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generates Kotlin code from the given IR tree.
    #[must_use]
    pub fn generate(&self, tree: &IrTree) -> String {
        let mut w = CodeWriter::new(4);
        w.write_line("import androidx.compose.foundation.layout.*");
        w.write_line("import androidx.compose.material3.*");
        w.write_line("import androidx.compose.runtime.Composable");
        w.write_line("import androidx.compose.ui.Modifier");
        w.write_line("import androidx.compose.ui.graphics.Color");
        w.write_line("import androidx.compose.ui.unit.dp");
        w.write_line("import androidx.compose.ui.text.font.FontWeight");
        w.blank_line();

        if let Some(root) = tree.nodes.iter().find(|n| n.id == tree.root_id) {
            self.emit_root(tree, root, &mut w);
        }

        w.into_string()
    }

    /// Emits the root composable function.
    fn emit_root(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("@Composable");
        w.write_line("fun GeneratedPage() {");
        w.indent();
        if node.children.is_empty() {
            w.write_line("// empty page");
        } else {
            // Wrap children in a Column by default
            w.write_line("Column(modifier = Modifier.fillMaxSize()) {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        }
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
            SemanticIr::NavigationBar => self.emit_app_bar(tree, node, w),
            SemanticIr::HeroSection => self.emit_hero(tree, node, w),
            SemanticIr::Card => self.emit_card(tree, node, w),
            SemanticIr::Button => self.emit_button(tree, node, w),
            SemanticIr::Text | SemanticIr::Paragraph => self.emit_text(tree, node, w),
            SemanticIr::Heading { level } => self.emit_heading(tree, node, *level, w),
            SemanticIr::Image => self.emit_image(tree, node, w),
            SemanticIr::Icon => self.emit_icon(tree, node, w),
            SemanticIr::Input | SemanticIr::TextArea => self.emit_text_field(tree, node, w),
            SemanticIr::Select => self.emit_dropdown(tree, node, w),
            SemanticIr::Checkbox => self.emit_checkbox(tree, node, w),
            SemanticIr::Radio => self.emit_radio(tree, node, w),
            SemanticIr::Form => self.emit_form(tree, node, w),
            SemanticIr::List | SemanticIr::LazyList => self.emit_lazy_column(tree, node, w),
            SemanticIr::ListItem => self.emit_list_item(tree, node, w),
            SemanticIr::Table => self.emit_table(tree, node, w),
            SemanticIr::TableRow | SemanticIr::TableCell => {
                self.emit_children(tree, &node.children, w);
            }
            SemanticIr::Row => self.emit_row(tree, node, w),
            SemanticIr::Column => self.emit_column(tree, node, w),
            SemanticIr::Stack => self.emit_box(tree, node, w),
            SemanticIr::Scroll => self.emit_scroll_column(tree, node, w),
            SemanticIr::Grid => self.emit_grid(tree, node, w),
            SemanticIr::Flex => self.emit_flex(tree, node, w),
            SemanticIr::Divider => w.write_line("HorizontalDivider()"),
            SemanticIr::Spacer => w.write_line("Spacer(modifier = Modifier.height(8.dp))"),
            SemanticIr::Dialog => self.emit_dialog(tree, node, w),
            SemanticIr::Tooltip => self.emit_tooltip(tree, node, w),
            SemanticIr::Badge => self.emit_badge(tree, node, w),
            SemanticIr::IconButton => self.emit_icon_button(tree, node, w),
            SemanticIr::Chip => {
                w.write_line("AssistChip(onClick = { }, label = { Text(\"chip\") })")
            }
            SemanticIr::Avatar => self.emit_avatar(tree, node, w),
            SemanticIr::Progress => w.write_line("LinearProgressIndicator()"),
            SemanticIr::Skeleton => {
                w.write_line("Box(modifier = Modifier.fillMaxWidth().height(20.dp).padding(4.dp))")
            }
            SemanticIr::Custom(name) => {
                w.write_line(&format!("// Custom composable: {name}"));
            }
        }
    }

    /// Emits a container, choosing Box/Row/Column based on layout.
    fn emit_container(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        let (widget, _modifier_call) = match node.layout {
            LayoutIr::FlexRow => ("Row", "Modifier"),
            LayoutIr::FlexColumn => ("Column", "Modifier"),
            _ => ("Box", "Modifier"),
        };
        let modifier = self.build_modifier(node);
        if modifier.is_empty() {
            w.write_line(&format!("{widget}() {{"));
        } else {
            w.write_line(&format!("{widget}(modifier = {modifier}) {{"));
        }
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Builds a Compose Modifier chain from computed style.
    fn build_modifier(&self, node: &IrNode) -> String {
        let style = &node.computed_style;
        let mut parts: Vec<String> = Vec::new();

        let p = &style.padding;
        if !self.is_zero_edge(p) {
            parts.push(self.format_padding_compose(p));
        }
        let m = &style.margin;
        if !self.is_zero_edge(m) {
            parts.push(self.format_margin_compose(m));
        }
        if let Some(ref color) = style.color {
            parts.push(format!("background({})", self.format_color_compose(color)));
        }
        if let Some(ref w_val) = style.width {
            if let Ok(px) = w_val.trim_end_matches("px").trim().parse::<f64>() {
                parts.push(format!("width({px}.dp)"));
            }
        }
        if let Some(ref h_val) = style.height {
            if let Ok(px) = h_val.trim_end_matches("px").trim().parse::<f64>() {
                parts.push(format!("height({px}.dp)"));
            }
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("Modifier.{}", parts.join("."))
        }
    }

    /// Returns true if all edge values are zero.
    fn is_zero_edge(&self, ev: &EdgeValues) -> bool {
        (ev.top - 0.0).abs() < f64::EPSILON
            && (ev.right - 0.0).abs() < f64::EPSILON
            && (ev.bottom - 0.0).abs() < f64::EPSILON
            && (ev.left - 0.0).abs() < f64::EPSILON
    }

    /// Formats padding as a Modifier.padding call.
    fn format_padding_compose(&self, ev: &EdgeValues) -> String {
        let top = ev.top;
        let right = ev.right;
        let bottom = ev.bottom;
        let left = ev.left;

        if (top - right).abs() < f64::EPSILON
            && (top - bottom).abs() < f64::EPSILON
            && (top - left).abs() < f64::EPSILON
        {
            format!("padding({top}.dp)")
        } else if (top - bottom).abs() < f64::EPSILON && (right - left).abs() < f64::EPSILON {
            format!("padding(vertical = {top}.dp, horizontal = {right}.dp)")
        } else {
            format!(
                "padding(start = {left}.dp, top = {top}.dp, end = {right}.dp, bottom = {bottom}.dp)"
            )
        }
    }

    /// Formats margin as a width/height spacer. Compose uses modifiers, not margin.
    fn format_margin_compose(&self, ev: &EdgeValues) -> String {
        let horizontal = ev.left + ev.right;
        let vertical = ev.top + ev.bottom;
        if (horizontal - vertical).abs() < f64::EPSILON {
            format!("padding({horizontal}.dp)")
        } else {
            format!(
                "padding(start = {}.dp, top = {}.dp, end = {}.dp, bottom = {}.dp)",
                ev.left, ev.top, ev.right, ev.bottom
            )
        }
    }

    /// Converts a CSS color string to Compose Color expression.
    fn format_color_compose(&self, color: &str) -> String {
        let hex = color.trim_start_matches('#');
        let padded = if hex.len() <= 6 {
            format!("{hex:0>6}")
        } else {
            hex.to_string()
        };
        format!("Color(0xFF{padded})")
    }

    /// Emits a Scaffold-like navigation structure.
    fn emit_navigation(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Scaffold(");
        w.indent();
        let has_bar = node.children.iter().any(|cid| {
            tree.nodes
                .iter()
                .any(|n| n.id == *cid && n.semantic == SemanticIr::NavigationBar)
        });
        if has_bar {
            w.write_line("topBar = {");
            w.indent();
            w.write_line("TopAppBar(title = { Text(\"Navigation\") })");
            w.dedent();
            w.write_line("},");
        }
        w.write_line("content = { paddingValues ->");
        w.indent();
        w.write_line("Column(modifier = Modifier.padding(paddingValues)) {");
        w.indent();
        let body_ids: Vec<NodeId> = node
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
        self.emit_children_for_ids(tree, &body_ids, w);
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("},");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a `TopAppBar`.
    fn emit_app_bar(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("TopAppBar(");
        w.indent();
        w.write_line("title = { Text(\"Page\") }");
        w.dedent();
        w.write_line(")");
    }

    /// Emits a Hero section.
    fn emit_hero(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Box(");
        w.indent();
        w.write_line("modifier = Modifier.fillMaxWidth().padding(24.dp),");
        if let Some(ref color) = node.computed_style.color {
            w.write_line(&format!(
                "background = {},",
                self.format_color_compose(color)
            ));
        }
        w.write_line("contentAlignment = Alignment.Center");
        w.dedent();
        w.write_line(") {");
        w.indent();
        w.write_line("Column {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a Card composable.
    fn emit_card(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Card(");
        w.indent();
        w.write_line("modifier = Modifier.padding(8.dp)");
        w.dedent();
        w.write_line(") {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a Button composable.
    fn emit_button(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        if node.children.is_empty() {
            w.write_line("Button(onClick = { }) {");
            w.indent();
            w.write_line("Text(\"Button\")");
            w.dedent();
            w.write_line("}");
        } else {
            w.write_line("Button(onClick = { }) {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        }
    }

    /// Emits a Text composable.
    fn emit_text(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Text(\"Text content\")");
    }

    /// Emits a heading as styled Text.
    fn emit_heading(&self, _tree: &IrTree, _node: &IrNode, level: u32, w: &mut CodeWriter) {
        let size = match level {
            1 => "32",
            2 => "28",
            3 => "24",
            4 => "20",
            5 => "18",
            _ => "16",
        };
        w.write_line(&format!(
            "Text(text = \"Heading {level}\", fontSize = {size}.sp, fontWeight = FontWeight.Bold)"
        ));
    }

    /// Emits an Image composable.
    fn emit_image(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("// Image composable - requires coil or similar library");
        w.write_line("Image(");
        w.indent();
        w.write_line("painter = rememberImagePainter(\"https://example.com/image.png\"),");
        w.write_line("contentDescription = \"Image\"");
        w.dedent();
        w.write_line(")");
    }

    /// Emits an Icon composable.
    fn emit_icon(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Icon(");
        w.indent();
        w.write_line("imageVector = Icons.Default.Star,");
        w.write_line("contentDescription = \"Star\"");
        w.dedent();
        w.write_line(")");
    }

    /// Emits a `TextField` composable.
    fn emit_text_field(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("OutlinedTextField(");
        w.indent();
        w.write_line("value = \"\",");
        w.write_line("onValueChange = { },");
        w.write_line("label = { Text(\"Input\") }");
        w.dedent();
        w.write_line(")");
    }

    /// Emits a `DropdownMenu` composable.
    fn emit_dropdown(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("DropdownMenu(");
        w.indent();
        w.write_line("expanded = false,");
        w.write_line("onDismissRequest = { }) {");
        w.indent();
        w.write_line("DropdownMenuItem(text = { Text(\"Option\") }, onClick = { })");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a Checkbox composable.
    fn emit_checkbox(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Checkbox(");
        w.indent();
        w.write_line("checked = false,");
        w.write_line("onCheckedChange = { }");
        w.dedent();
        w.write_line(")");
    }

    /// Emits a `RadioButton` composable.
    fn emit_radio(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("RadioButton(");
        w.indent();
        w.write_line("selected = false,");
        w.write_line("onClick = { }");
        w.dedent();
        w.write_line(")");
    }

    /// Emits a Form layout.
    fn emit_form(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Column(modifier = Modifier.padding(16.dp)) {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a `LazyColumn` (lazy list).
    fn emit_lazy_column(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("LazyColumn {");
        w.indent();
        w.write_line("items(listOf(1)) {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a list item.
    fn emit_list_item(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("ListItem(");
        w.indent();
        if node.children.is_empty() {
            w.write_line("headlineContent = { Text(\"Item\") }");
        } else {
            w.write_line("headlineContent = {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        }
        w.dedent();
        w.write_line(")");
    }

    /// Emits a table layout.
    fn emit_table(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("// Table layout not directly available in Compose");
        w.write_line("// Consider using LazyVerticalGrid or custom layout");
    }

    /// Emits a Row composable.
    fn emit_row(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Row(");
        w.indent();
        let style = &node.computed_style;
        if let Some(ref jc) = style.justify_content {
            let value = match jc {
                motarjim_ast_html::JustifyContent::FlexStart => "Arrangement.Start",
                motarjim_ast_html::JustifyContent::FlexEnd => "Arrangement.End",
                motarjim_ast_html::JustifyContent::Center => "Arrangement.Center",
                motarjim_ast_html::JustifyContent::SpaceBetween => "Arrangement.SpaceBetween",
                motarjim_ast_html::JustifyContent::SpaceAround => "Arrangement.SpaceAround",
                motarjim_ast_html::JustifyContent::SpaceEvenly => "Arrangement.SpaceEvenly",
            };
            w.write_line(&format!("horizontalArrangement = {value},"));
        }
        if let Some(ref ai) = style.align_items {
            let value = match ai {
                motarjim_ast_html::AlignItems::FlexStart => "Alignment.Top",
                motarjim_ast_html::AlignItems::FlexEnd => "Alignment.Bottom",
                motarjim_ast_html::AlignItems::Center => "Alignment.CenterVertically",
                motarjim_ast_html::AlignItems::Stretch => "Alignment.Stretch",
                _ => "Alignment.Top",
            };
            w.write_line(&format!("verticalAlignment = {value},"));
        }
        w.dedent();
        w.write_line(") {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a Column composable.
    fn emit_column(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Column(");
        w.indent();
        let style = &node.computed_style;
        if let Some(ref ai) = style.align_items {
            let value = match ai {
                motarjim_ast_html::AlignItems::FlexStart => "Alignment.Start",
                motarjim_ast_html::AlignItems::FlexEnd => "Alignment.End",
                motarjim_ast_html::AlignItems::Center => "Alignment.CenterHorizontally",
                motarjim_ast_html::AlignItems::Stretch => "Alignment.Stretch",
                _ => "Alignment.Start",
            };
            w.write_line(&format!("horizontalAlignment = {value},"));
        }
        w.dedent();
        w.write_line(") {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a Box composable (Stack equivalent).
    fn emit_box(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Box(modifier = Modifier.fillMaxSize()) {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a scrollable column.
    fn emit_scroll_column(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Column(");
        w.indent();
        w.write_line("modifier = Modifier.verticalScroll(rememberScrollState())");
        w.dedent();
        w.write_line(") {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
    }

    /// Emits a grid layout.
    fn emit_grid(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("LazyVerticalGrid(");
        w.indent();
        w.write_line("columns = GridCells.Fixed(2),");
        w.write_line("modifier = Modifier.fillMaxSize()");
        w.dedent();
        w.write_line(") {");
        w.indent();
        w.write_line("items(listOf(1)) {");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("}");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a Flex layout.
    fn emit_flex(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        if node.layout == LayoutIr::FlexColumn {
            w.write_line("Column {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        } else {
            w.write_line("Row {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        }
    }

    /// Emits a dialog.
    fn emit_dialog(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("AlertDialog(");
        w.indent();
        w.write_line("onDismissRequest = { },");
        w.write_line("title = { Text(\"Dialog\") },");
        if !node.children.is_empty() {
            w.write_line("text = {");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("}");
        }
        w.dedent();
        w.write_line(")");
    }

    /// Emits a tooltip.
    fn emit_tooltip(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("// Tooltip - use PlainTooltipBox or similar");
        w.write_line("Box {");
        w.indent();
        w.write_line("Text(\"Hover me\")");
        w.dedent();
        w.write_line("}");
    }

    /// Emits a badge.
    fn emit_badge(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("BadgedBox(badge = { Badge { Text(\"1\") } }) {");
        w.indent();
        w.write_line("Icon(Icons.Default.Notifications, contentDescription = \"Notifications\")");
        w.dedent();
        w.write_line("}");
    }

    /// Emits an icon button.
    fn emit_icon_button(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("IconButton(onClick = { }) {");
        w.indent();
        w.write_line("Icon(Icons.Default.Star, contentDescription = \"Star\")");
        w.dedent();
        w.write_line("}");
    }

    /// Emits an avatar.
    fn emit_avatar(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("// Avatar - use a circle ClipShape");
        w.write_line("Surface(");
        w.indent();
        w.write_line("shape = CircleShape,");
        w.write_line("modifier = Modifier.size(48.dp),");
        w.write_line("color = MaterialTheme.colorScheme.primary");
        w.dedent();
        w.write_line(") { }");
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
}

impl Default for ComposeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
