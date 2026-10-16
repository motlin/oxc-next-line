use oxc_allocator::Allocator;
use oxc_formatter_core::LineWidth;
use oxc_formatter_graphql::{BraceStyle, GraphqlFormatOptions, format};

fn format_with(source_text: &str, brace_style: BraceStyle, line_width: u16) -> String {
    let allocator = Allocator::default();
    let options = GraphqlFormatOptions {
        brace_style,
        line_width: LineWidth::try_from(line_width).unwrap(),
        ..GraphqlFormatOptions::default()
    };
    format(&allocator, source_text, options).unwrap().print().unwrap().into_code()
}

const QUERY: &str = "query Example { viewer { settings(value: { enabled: true }) { id } } }";

#[test]
fn same_line_is_the_default_and_unchanged() {
    assert_eq!(GraphqlFormatOptions::default().brace_style, BraceStyle::SameLine);
    assert_eq!(
        format_with(QUERY, BraceStyle::SameLine, 80),
        "query Example {\n  viewer {\n    settings(value: { enabled: true }) {\n      id\n    }\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_selection_set_braces_on_the_next_line_and_keeps_fitting_objects_inline() {
    assert_eq!(
        format_with(QUERY, BraceStyle::NextLine, 80),
        "query Example\n{\n  viewer\n  {\n    settings(value: { enabled: true })\n    {\n      id\n    }\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_wrapped_object_value_braces_on_the_next_line() {
    assert_eq!(
        format_with(
            "query { viewer(value: { enabledddddd: true, otherrrrrrr: false }) { id } }",
            BraceStyle::NextLine,
            40,
        ),
        "query\n{\n  viewer(\n    value:\n    {\n      enabledddddd: true\n      otherrrrrrr: false\n    }\n  )\n  {\n    id\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_definition_braces_on_the_next_line() {
    assert_eq!(
        format_with(
            "schema { query: Query }\n\ntype Query { viewer: User }\n\nenum E { A B }\n\nextend schema { mutation: M }",
            BraceStyle::NextLine,
            80,
        ),
        "schema\n{\n  query: Query\n}\n\ntype Query\n{\n  viewer: User\n}\n\nenum E\n{\n  A\n  B\n}\n\nextend schema\n{\n  mutation: M\n}\n",
    );
}

#[test]
fn next_line_puts_wrapped_default_object_value_braces_on_the_next_line() {
    assert_eq!(
        format_with(
            "query ($v: In = { enabledddddd: true, otherrrrrrr: false }) { id }",
            BraceStyle::NextLine,
            40,
        ),
        "query (\n  $v: In =\n  {\n    enabledddddd: true\n    otherrrrrrr: false\n  }\n)\n{\n  id\n}\n",
    );
}
