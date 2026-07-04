use crate::*;

/// Generates Flutter/Dart code from an IR tree.
#[derive(Debug, Clone)]
pub struct FlutterGenerator;

#[allow(clippy::unused_self)]
impl FlutterGenerator {
    /// Creates a new `FlutterGenerator`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generates Dart code from the given IR tree.
    #[must_use]
    pub fn generate(&self, tree: &IrTree) -> String {
        let mut w = CodeWriter::new(2);
        w.write_line("import 'package:flutter/material.dart';");
        w.blank_line();

        if let Some(root) = tree.nodes.iter().find(|n| n.id == tree.root_id) {
            self.emit_root(tree, root, &mut w);
        }

        w.into_string()
    }

    /// Emits the root widget structure.
    fn emit_root(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("class GeneratedPage extends StatelessWidget {");
        w.indent();
        w.write_line("const GeneratedPage({super.key});");
        w.blank_line();
        w.write_line("@override");
        w.write_line("Widget build(BuildContext context) {");
        w.indent();
        if node.children.is_empty() {
            w.write_line("return const SizedBox.shrink();");
        } else {
            w.write_line("return Scaffold(");
            w.indent();
            w.write_line("body: ");
            w.indent();
            self.emit_children(tree, &node.children, &mut *w);
            w.dedent();
            w.dedent();
            w.write_line(");");
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
            SemanticIr::List | SemanticIr::LazyList => self.emit_list_view(tree, node, w),
            SemanticIr::ListItem => self.emit_list_item(tree, node, w),
            SemanticIr::Table => self.emit_table(tree, node, w),
            SemanticIr::TableRow => self.emit_table_row(tree, node, w),
            SemanticIr::TableCell => self.emit_table_cell(tree, node, w),
            SemanticIr::Row => self.emit_row(tree, node, w),
            SemanticIr::Column => self.emit_column(tree, node, w),
            SemanticIr::Stack => self.emit_stack(tree, node, w),
            SemanticIr::Scroll => self.emit_scroll_view(tree, node, w),
            SemanticIr::Grid => self.emit_grid(tree, node, w),
            SemanticIr::Flex => self.emit_flex(tree, node, w),
            SemanticIr::Divider => w.write_line("const Divider(),"),
            SemanticIr::Spacer => w.write_line("const Spacer(),"),
            SemanticIr::Dialog => self.emit_dialog(tree, node, w),
            SemanticIr::Tooltip => self.emit_tooltip(tree, node, w),
            SemanticIr::Badge => self.emit_badge(tree, node, w),
            SemanticIr::IconButton => self.emit_icon_button(tree, node, w),
            SemanticIr::Chip => w.write_line("const Chip(label: Text('chip')),"),
            SemanticIr::Avatar => self.emit_avatar(tree, node, w),
            SemanticIr::Progress => w.write_line("const LinearProgressIndicator(),"),
            SemanticIr::Skeleton => {
                w.write_line("const SizedBox(width: double.infinity, height: 20),");
            }
            SemanticIr::Custom(name) => {
                w.write_line(&format!("// Custom widget: {name}"));
            }
        }
    }

    /// Emits a container widget, choosing Row/Column/Container based on layout.
    fn emit_container(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        let widget = match node.layout {
            LayoutIr::FlexRow => "Row",
            LayoutIr::FlexColumn => "Column",
            _ => "Container",
        };
        w.write_line(&format!("{widget}("));
        w.indent();

        let style = &node.computed_style;
        self.emit_edge_param(w, "padding", &style.padding);
        self.emit_edge_param(w, "margin", &style.margin);

        if let Some(ref color) = style.color {
            w.write_line(&format!("color: {},", self.format_color_dart(color)));
        }

        if widget == "Container" {
            if node.children.len() == 1 {
                w.write_line("child: ");
                w.indent();
                self.emit_children(tree, &node.children, w);
                w.dedent();
            } else if !node.children.is_empty() {
                w.write_line("child: Column(");
                w.indent();
                w.write_line("children: [");
                w.indent();
                self.emit_children(tree, &node.children, w);
                w.dedent();
                w.write_line("],");
                w.dedent();
                w.write_line("),");
            }
        } else if !node.children.is_empty() {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }

        w.dedent();
        w.write_line("),");
    }

    /// Emits a navigation structure with drawer/AppBar.
    fn emit_navigation(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Scaffold(");
        w.indent();
        // Look for a NavigationBar child and emit as AppBar
        let has_app_bar = node.children.iter().any(|cid| {
            tree.nodes
                .iter()
                .any(|n| n.id == *cid && n.semantic == SemanticIr::NavigationBar)
        });
        if has_app_bar {
            w.write_line("appBar: AppBar(");
            w.indent();
            w.write_line("title: const Text('Navigation'),");
            w.dedent();
            w.write_line("),");
        }
        if !node.children.is_empty() {
            let body_children: Vec<NodeId> = node
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
            if !body_children.is_empty() {
                w.write_line("body: ");
                w.indent();
                self.emit_children_for_ids(tree, &body_children, w);
                w.dedent();
            }
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits an `AppBar` widget.
    fn emit_app_bar(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("AppBar(");
        w.indent();
        // Look for text children as title
        let title_text = node.children.iter().find_map(|cid| {
            tree.nodes.iter().find(|n| {
                n.id == *cid && matches!(n.semantic, SemanticIr::Text | SemanticIr::Heading { .. })
            })
        });
        if title_text.is_some() {
            w.write_line("title: ");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
        } else {
            w.write_line("title: const Text('Page'),");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Hero section widget.
    fn emit_hero(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("SizedBox(");
        w.indent();
        w.write_line("width: double.infinity,");
        w.write_line("child: Container(");
        w.indent();
        if let Some(ref color) = node.computed_style.color {
            w.write_line(&format!("color: {},", self.format_color_dart(color)));
        }
        w.write_line("padding: const EdgeInsets.all(24.0),");
        w.write_line("child: Column(");
        w.indent();
        w.write_line("children: [");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.write_line("],");
        w.dedent();
        w.write_line("),");
        w.dedent();
        w.write_line("),");
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Card widget.
    fn emit_card(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Card(");
        w.indent();
        if !node.children.is_empty() {
            w.write_line("child: ");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits an `ElevatedButton`.
    fn emit_button(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("ElevatedButton(");
        w.indent();
        w.write_line("onPressed: () {},");
        if node.children.is_empty() {
            w.write_line("child: const Text('Button'),");
        } else {
            w.write_line("child: ");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Text widget.
    fn emit_text(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("const Text('Text content'),");
    }

    /// Emits a heading as a styled Text widget.
    fn emit_heading(&self, _tree: &IrTree, _node: &IrNode, level: u32, w: &mut CodeWriter) {
        let size = match level {
            1 => "32.0",
            2 => "28.0",
            3 => "24.0",
            4 => "20.0",
            5 => "18.0",
            _ => "16.0",
        };
        w.write_line("Text(");
        w.indent();
        w.write_line(&format!("'Heading {level}',"));
        w.write_line(&format!(
            "style: TextStyle(fontSize: {size}, fontWeight: FontWeight.bold),"
        ));
        w.dedent();
        w.write_line("),");
    }

    /// Emits an Image widget.
    fn emit_image(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Image.network(");
        w.indent();
        w.write_line("'https://example.com/image.png',");
        w.dedent();
        w.write_line("),");
    }

    /// Emits an Icon widget.
    fn emit_icon(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("const Icon(Icons.star),");
    }

    /// Emits a `TextField` widget.
    fn emit_text_field(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("const TextField(");
        w.indent();
        w.write_line("decoration: InputDecoration(");
        w.indent();
        w.write_line("border: OutlineInputBorder(),");
        w.dedent();
        w.write_line("),");
        w.dedent();
        w.write_line("),");
    }

    /// Emits a `DropdownButton` widget.
    fn emit_dropdown(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("const DropdownButton<String>(");
        w.indent();
        w.write_line("items: [],");
        w.write_line("onChanged: null,");
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Checkbox widget.
    fn emit_checkbox(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("const Checkbox(");
        w.indent();
        w.write_line("value: false,");
        w.write_line("onChanged: null,");
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Radio widget.
    fn emit_radio(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("const Radio<int>(");
        w.indent();
        w.write_line("value: 0,");
        w.write_line("groupValue: null,");
        w.write_line("onChanged: null,");
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Form widget.
    fn emit_form(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Form(");
        w.indent();
        if !node.children.is_empty() {
            w.write_line("child: Column(");
            w.indent();
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
            w.dedent();
            w.write_line("),");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a `ListView` widget.
    fn emit_list_view(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("ListView(");
        w.indent();
        if node.children.is_empty() {
            w.write_line("children: const <Widget>[],");
        } else {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a `ListItem` widget.
    fn emit_list_item(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("ListTile(");
        w.indent();
        if node.children.is_empty() {
            w.write_line("title: const Text('Item'),");
        } else {
            w.write_line("title: ");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Table widget.
    fn emit_table(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Table(");
        w.indent();
        w.write_line("border: TableBorder.all(),");
        if !node.children.is_empty() {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a `TableRow` widget.
    fn emit_table_row(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("TableRow(");
        w.indent();
        if !node.children.is_empty() {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a `TableCell` widget.
    fn emit_table_cell(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("TableRow(");
        w.indent();
        w.write_line("child: ");
        w.indent();
        self.emit_children(tree, &node.children, w);
        w.dedent();
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Row widget.
    fn emit_row(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Row(");
        w.indent();
        let style = &node.computed_style;
        self.emit_main_axis_alignment(w, &style.justify_content);
        self.emit_cross_axis_alignment(w, &style.align_items);
        if node.children.is_empty() {
            w.write_line("children: const <Widget>[],");
        } else {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Column widget.
    fn emit_column(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Column(");
        w.indent();
        let style = &node.computed_style;
        self.emit_main_axis_alignment(w, &style.justify_content);
        self.emit_cross_axis_alignment(w, &style.align_items);
        if node.children.is_empty() {
            w.write_line("children: const <Widget>[],");
        } else {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits alignment parameters for main axis.
    fn emit_main_axis_alignment(
        &self,
        w: &mut CodeWriter,
        justify: &Option<motarjim_ast::style::JustifyContent>,
    ) {
        if let Some(ref jc) = justify {
            let value = match jc {
                motarjim_ast::style::JustifyContent::FlexStart => "MainAxisAlignment.start",
                motarjim_ast::style::JustifyContent::FlexEnd => "MainAxisAlignment.end",
                motarjim_ast::style::JustifyContent::Center => "MainAxisAlignment.center",
                motarjim_ast::style::JustifyContent::SpaceBetween => {
                    "MainAxisAlignment.spaceBetween"
                }
                motarjim_ast::style::JustifyContent::SpaceAround => "MainAxisAlignment.spaceAround",
                motarjim_ast::style::JustifyContent::SpaceEvenly => "MainAxisAlignment.spaceEvenly",
            };
            w.write_line(&format!("mainAxisAlignment: {value},"));
        }
    }

    /// Emits alignment parameters for cross axis.
    fn emit_cross_axis_alignment(
        &self,
        w: &mut CodeWriter,
        align: &Option<motarjim_ast::style::AlignItems>,
    ) {
        if let Some(ref ai) = align {
            let value = match ai {
                motarjim_ast::style::AlignItems::FlexStart => "CrossAxisAlignment.start",
                motarjim_ast::style::AlignItems::FlexEnd => "CrossAxisAlignment.end",
                motarjim_ast::style::AlignItems::Center => "CrossAxisAlignment.center",
                motarjim_ast::style::AlignItems::Stretch => "CrossAxisAlignment.stretch",
                motarjim_ast::style::AlignItems::Baseline => "CrossAxisAlignment.baseline",
            };
            w.write_line(&format!("crossAxisAlignment: {value},"));
        }
    }

    /// Emits a Stack widget.
    fn emit_stack(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Stack(");
        w.indent();
        if node.children.is_empty() {
            w.write_line("children: const <Widget>[],");
        } else {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a `ScrollView` widget.
    fn emit_scroll_view(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("SingleChildScrollView(");
        w.indent();
        if !node.children.is_empty() {
            w.write_line("child: Column(");
            w.indent();
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
            w.dedent();
            w.write_line("),");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Grid widget.
    fn emit_grid(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("GridView(");
        w.indent();
        w.write_line("gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(");
        w.indent();
        w.write_line("crossAxisCount: 2,");
        w.dedent();
        w.write_line("),");
        if !node.children.is_empty() {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Flex widget, choosing direction based on layout.
    fn emit_flex(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        let direction = match node.layout {
            LayoutIr::FlexColumn => "Axis.vertical",
            _ => "Axis.horizontal",
        };
        w.write_line("Flex(");
        w.indent();
        w.write_line(&format!("direction: {direction},"));
        if !node.children.is_empty() {
            w.write_line("children: [");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
            w.write_line("],");
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Dialog widget.
    fn emit_dialog(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("AlertDialog(");
        w.indent();
        w.write_line("title: const Text('Dialog'),");
        if !node.children.is_empty() {
            w.write_line("content: ");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Tooltip widget.
    fn emit_tooltip(&self, tree: &IrTree, node: &IrNode, w: &mut CodeWriter) {
        w.write_line("Tooltip(");
        w.indent();
        w.write_line("message: 'Tooltip',");
        if node.children.is_empty() {
            w.write_line("child: const Icon(Icons.info),");
        } else {
            w.write_line("child: ");
            w.indent();
            self.emit_children(tree, &node.children, w);
            w.dedent();
        }
        w.dedent();
        w.write_line("),");
    }

    /// Emits a Badge widget.
    fn emit_badge(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("const Badge(");
        w.indent();
        w.write_line("label: Text('1'),");
        w.dedent();
        w.write_line("),");
    }

    /// Emits an `IconButton` widget.
    fn emit_icon_button(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("IconButton(");
        w.indent();
        w.write_line("onPressed: () {},");
        w.write_line("icon: const Icon(Icons.star),");
        w.dedent();
        w.write_line("),");
    }

    /// Emits an Avatar widget.
    fn emit_avatar(&self, _tree: &IrTree, _node: &IrNode, w: &mut CodeWriter) {
        w.write_line("const CircleAvatar(");
        w.indent();
        w.write_line("radius: 24.0,");
        w.dedent();
        w.write_line("),");
    }

    /// Emits all children of a node.
    fn emit_children(&self, tree: &IrTree, children: &[NodeId], w: &mut CodeWriter) {
        for child_id in children {
            if let Some(child) = tree.nodes.iter().find(|n| n.id == *child_id) {
                self.emit_widget(tree, child, w);
            }
        }
    }

    /// Emits all given child IDs as a sequence.
    fn emit_children_for_ids(&self, tree: &IrTree, ids: &[NodeId], w: &mut CodeWriter) {
        for child_id in ids {
            if let Some(child) = tree.nodes.iter().find(|n| n.id == *child_id) {
                self.emit_widget(tree, child, w);
            }
        }
    }

    /// Emits an `EdgeValues` parameter like `padding` or `margin`.
    fn emit_edge_param(&self, w: &mut CodeWriter, name: &str, ev: &EdgeValues) {
        if (ev.top - 0.0).abs() < f64::EPSILON
            && (ev.right - 0.0).abs() < f64::EPSILON
            && (ev.bottom - 0.0).abs() < f64::EPSILON
            && (ev.left - 0.0).abs() < f64::EPSILON
        {
            return;
        }
        w.write_line(&format!("{}: {},", name, self.format_edge_values_dart(ev)));
    }

    /// Formats `EdgeValues` as a Dart `EdgeInsets` expression.
    fn format_edge_values_dart(&self, ev: &EdgeValues) -> String {
        let top = ev.top;
        let right = ev.right;
        let bottom = ev.bottom;
        let left = ev.left;

        if (top - right).abs() < f64::EPSILON
            && (top - bottom).abs() < f64::EPSILON
            && (top - left).abs() < f64::EPSILON
        {
            if (top - 0.0).abs() < f64::EPSILON {
                String::from("EdgeInsets.zero")
            } else {
                format!("const EdgeInsets.all({top})")
            }
        } else if (top - bottom).abs() < f64::EPSILON && (right - left).abs() < f64::EPSILON {
            format!("const EdgeInsets.symmetric(vertical: {top}, horizontal: {right})")
        } else {
            format!(
                "const EdgeInsets.only(top: {top}, right: {right}, bottom: {bottom}, left: {left})"
            )
        }
    }

    /// Converts a CSS color string to a Dart Color expression.
    fn format_color_dart(&self, color: &str) -> String {
        let hex = color.trim_start_matches('#');
        // Pad to 6 or 8 hex digits
        let padded = if hex.len() <= 6 {
            format!("{hex:0>6}")
        } else {
            hex.to_string()
        };
        format!("Color(0xFF{padded})")
    }
}

impl Default for FlutterGenerator {
    fn default() -> Self {
        Self::new()
    }
}
