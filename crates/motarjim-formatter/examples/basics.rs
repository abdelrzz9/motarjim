use motarjim_formatter::CodeWriter;

fn main() {
    let mut w = CodeWriter::new(4);

    w.write_line("Widget build(BuildContext context) {");
    w.indent();
    w.write_line("return Column(");
    w.indent();
    w.write_line("children: [");
    w.indent();
    w.write_line("Text(\"Hello\"),");
    w.write_line("Text(\"World\"),");
    w.dedent();
    w.write_line("],");
    w.dedent();
    w.write_line(");");
    w.dedent();
    w.write_line("}");

    println!("{}", w.as_str());
    println!("---");
    println!("Total chars: {}", w.into_string().len());
}
