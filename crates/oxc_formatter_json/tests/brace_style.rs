use oxc_allocator::Allocator;
use oxc_formatter_json::{BraceStyle, JsonFormatOptions, JsonVariant, format};

fn format_with(source_text: &str, variant: JsonVariant, brace_style: BraceStyle) -> String {
    let allocator = Allocator::default();
    let options = JsonFormatOptions { variant, brace_style, ..JsonFormatOptions::default() };
    format(&allocator, source_text, options).unwrap().print().unwrap().into_code()
}

const NESTED: &str = "{\n\"outer\": {\n\"inner\": {\"a\": 1},\n\"list\": [{\n\"b\": 2}]\n}\n}";

#[test]
fn same_line_is_the_default_and_unchanged() {
    assert_eq!(JsonFormatOptions::default().brace_style, BraceStyle::SameLine);
    assert_eq!(
        format_with(NESTED, JsonVariant::Json, BraceStyle::SameLine),
        "{\n  \"outer\": {\n    \"inner\": { \"a\": 1 },\n    \"list\": [\n      {\n        \"b\": 2\n      }\n    ]\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_wrapped_object_braces_on_the_next_line_and_keeps_fitting_objects_inline() {
    assert_eq!(
        format_with(NESTED, JsonVariant::Json, BraceStyle::NextLine),
        "{\n  \"outer\":\n  {\n    \"inner\": { \"a\": 1 },\n    \"list\": [\n      {\n        \"b\": 2\n      }\n    ]\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_json_stringify_object_braces_on_the_next_line() {
    assert_eq!(
        format_with(
            r#"{"outer":{"empty":{},"a":1}}"#,
            JsonVariant::JsonStringify,
            BraceStyle::NextLine
        ),
        "{\n  \"outer\":\n  {\n    \"empty\": {},\n    \"a\": 1\n  }\n}\n",
    );
}
