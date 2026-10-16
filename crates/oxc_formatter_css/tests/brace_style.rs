use oxc_allocator::Allocator;
use oxc_formatter_css::{BraceStyle, CssFormatOptions, CssVariant, format};

fn format_with(source_text: &str, variant: CssVariant, brace_style: BraceStyle) -> String {
    let allocator = Allocator::default();
    let options = CssFormatOptions { variant, brace_style, ..CssFormatOptions::default() };
    format(&allocator, source_text, options).unwrap().print().unwrap().into_code()
}

#[test]
fn same_line_is_the_default_and_unchanged() {
    assert_eq!(CssFormatOptions::default().brace_style, BraceStyle::SameLine);
    assert_eq!(
        format_with(".alpha { color: red; }", CssVariant::Css, BraceStyle::SameLine),
        ".alpha {\n  color: red;\n}\n",
    );
}

#[test]
fn next_line_puts_rule_and_at_rule_braces_on_the_next_line() {
    assert_eq!(
        format_with(
            ".alpha { color: red; } @media (width > 1px) { .beta {} }",
            CssVariant::Css,
            BraceStyle::NextLine,
        ),
        ".alpha\n{\n  color: red;\n}\n@media (width > 1px)\n{\n  .beta\n  {\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_scss_nested_property_braces_on_the_next_line() {
    assert_eq!(
        format_with(".alpha { font: { size: 1px; } }", CssVariant::Scss, BraceStyle::NextLine),
        ".alpha\n{\n  font:\n  {\n    size: 1px;\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_less_mixin_braces_on_the_next_line() {
    assert_eq!(
        format_with(".mixin() { color: red; }", CssVariant::Less, BraceStyle::NextLine),
        ".mixin()\n{\n  color: red;\n}\n",
    );
}

#[test]
fn next_line_puts_scss_else_on_its_own_line() {
    assert_eq!(
        format_with("@if $a { b: c; } @else { d: e; }", CssVariant::Scss, BraceStyle::NextLine),
        "@if $a\n{\n  b: c;\n}\n@else\n{\n  d: e;\n}\n",
    );
}

#[test]
fn next_line_puts_custom_property_block_braces_on_the_next_line() {
    assert_eq!(
        format_with(":root { --prop: { a: b; c: d }; }", CssVariant::Css, BraceStyle::NextLine),
        ":root\n{\n  --prop:\n  {\n    a: b;\n    c: d;\n  };\n}\n",
    );
}
