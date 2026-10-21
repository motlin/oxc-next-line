use oxc_allocator::Allocator;
use oxc_formatter::{BraceStyle, JsFormatOptions, format};
use oxc_span::SourceType;

fn format_with(source_text: &str, source_type: SourceType, brace_style: BraceStyle) -> String {
    let allocator = Allocator::default();
    let options = JsFormatOptions { brace_style, ..JsFormatOptions::default() };
    format(&allocator, source_text, source_type, options).unwrap().print().unwrap().into_code()
}

const FUNCTION_WITH_IF_ELSE: &str =
    "function example(value) { if (value) { return true; } else { return false; } }";

#[test]
fn same_line_is_the_library_default_and_unchanged() {
    assert_eq!(JsFormatOptions::default().brace_style, BraceStyle::SameLine);
    assert_eq!(
        format_with(FUNCTION_WITH_IF_ELSE, SourceType::mjs(), BraceStyle::SameLine),
        "function example(value) {\n  if (value) {\n    return true;\n  } else {\n    return false;\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_function_and_if_else_braces_on_the_next_line() {
    assert_eq!(
        format_with(FUNCTION_WITH_IF_ELSE, SourceType::mjs(), BraceStyle::NextLine),
        "function example(value)\n{\n  if (value)\n  {\n    return true;\n  }\n  else\n  {\n    return false;\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_class_and_method_braces_on_the_next_line() {
    assert_eq!(
        format_with(
            "class Example { method(value: number) { return value; } }",
            SourceType::ts(),
            BraceStyle::NextLine,
        ),
        "class Example\n{\n  method(value: number)\n  {\n    return value;\n  }\n}\n",
    );
}

#[test]
fn next_line_puts_try_catch_finally_and_switch_braces_on_the_next_line() {
    assert_eq!(
        format_with(
            "try { a(); } catch (e) { b(); } finally { c(); } switch (x) { case 1: d(); }",
            SourceType::mjs(),
            BraceStyle::NextLine,
        ),
        "try\n{\n  a();\n}\ncatch (e)\n{\n  b();\n}\nfinally\n{\n  c();\n}\nswitch (x)\n{\n  case 1:\n    d();\n}\n",
    );
}

#[test]
fn next_line_puts_arrow_function_body_braces_on_the_next_line() {
    assert_eq!(
        format_with("const f = (a) => { return a; };", SourceType::mjs(), BraceStyle::NextLine),
        "const f = (a) =>\n{\n  return a;\n};\n",
    );
}

#[test]
fn next_line_keeps_returned_object_brace_attached() {
    assert_eq!(
        format_with(
            "function f() { return { enabled: true, other: false, third: 3, fourth: 4, fifth: 555555, sixth: \"a long string\" }; }",
            SourceType::mjs(),
            BraceStyle::NextLine,
        ),
        "function f()\n{\n  return {\n    enabled: true,\n    other: false,\n    third: 3,\n    fourth: 4,\n    fifth: 555555,\n    sixth: \"a long string\",\n  };\n}\n",
    );
}

fn format_next_line_narrow(source_text: &str, source_type: SourceType) -> String {
    let allocator = Allocator::default();
    let options = JsFormatOptions {
        brace_style: BraceStyle::NextLine,
        line_width: oxc_formatter_core::LineWidth::try_from(40).unwrap(),
        ..JsFormatOptions::default()
    };
    format(&allocator, source_text, source_type, options).unwrap().print().unwrap().into_code()
}

#[test]
fn next_line_keeps_braces_that_fit_on_one_line_inline() {
    assert_eq!(
        format_next_line_narrow(
            "import { a } from \"x\";\nconst value = { a: 1 };\nconst { b } = value;\ntype T = { c: string };",
            SourceType::ts(),
        ),
        "import { a } from \"x\";\nconst value = { a: 1 };\nconst { b } = value;\ntype T = { c: string };\n",
    );
}

#[test]
fn next_line_puts_wrapped_object_braces_on_the_next_line() {
    assert_eq!(
        format_next_line_narrow(
            "const value = { alpha: 1111111111, nested: { beta: 2222222222, gamma: 3333333333 } };",
            SourceType::mjs(),
        ),
        "const value =\n{\n  alpha: 1111111111,\n  nested:\n  {\n    beta: 2222222222,\n    gamma: 3333333333,\n  },\n};\n",
    );
}

#[test]
fn next_line_puts_wrapped_call_argument_object_braces_on_the_next_line() {
    assert_eq!(
        format_next_line_narrow(
            "call({ alpha: 1111111111, beta: 2222222222 });",
            SourceType::mjs()
        ),
        "call(\n  {\n    alpha: 1111111111,\n    beta: 2222222222,\n  },\n);\n",
    );
}

#[test]
fn next_line_puts_wrapped_import_braces_on_the_next_line() {
    assert_eq!(
        format_next_line_narrow(
            "import { alphaaaaaa, betaaaaaaa, gammaaaaa } from \"example\";",
            SourceType::mjs(),
        ),
        "import\n{\n  alphaaaaaa,\n  betaaaaaaa,\n  gammaaaaa,\n} from \"example\";\n",
    );
}

#[test]
fn next_line_puts_wrapped_destructuring_braces_on_the_next_line() {
    assert_eq!(
        format_next_line_narrow(
            "const { alphaaaaaa, betaaaaaaa, gammaaaaa } = value;",
            SourceType::mjs(),
        ),
        "const\n{\n  alphaaaaaa,\n  betaaaaaaa,\n  gammaaaaa,\n} = value;\n",
    );
}

#[test]
fn next_line_puts_wrapped_type_literal_braces_on_the_next_line() {
    assert_eq!(
        format_next_line_narrow(
            "type Settings = { enabled: boolean; otherOption: string };",
            SourceType::ts(),
        ),
        "type Settings =\n{\n  enabled: boolean;\n  otherOption: string;\n};\n",
    );
}

#[test]
fn next_line_keeps_wrapped_parameter_destructuring_hugged() {
    assert_eq!(
        format_next_line_narrow(
            "function f({ alphaaaaaa, betaaaaaaa, gammaaaaa }) { a(); }",
            SourceType::mjs(),
        ),
        "function f({\n  alphaaaaaa,\n  betaaaaaaa,\n  gammaaaaa,\n})\n{\n  a();\n}\n",
    );
}

#[test]
fn next_line_keeps_callbacks_hugged() {
    assert_eq!(
        format_next_line_narrow(
            "foo(() => { a(); }); foo(function () { a(); });",
            SourceType::mjs()
        ),
        "foo(() =>\n{\n  a();\n});\nfoo(function ()\n{\n  a();\n});\n",
    );
}

#[test]
fn next_line_puts_wrapped_export_braces_on_the_next_line() {
    assert_eq!(
        format_next_line_narrow(
            "export { alphaaaaaa, betaaaaaaa, gammaaaaa } from \"x\";",
            SourceType::mjs(),
        ),
        "export\n{\n  alphaaaaaa,\n  betaaaaaaa,\n  gammaaaaa,\n} from \"x\";\n",
    );
}

fn assert_next_line_is_idempotent(source_text: &str, source_type: SourceType, expected: &str) {
    let first_pass = format_with(source_text, source_type, BraceStyle::NextLine);
    let second_pass = format_with(&first_pass, source_type, BraceStyle::NextLine);
    assert_eq!(second_pass, first_pass);
    assert_eq!(first_pass, expected);
}

#[test]
fn next_line_is_idempotent_with_a_block_comment_before_a_type_literal_brace() {
    assert_next_line_is_idempotent(
        "type M1 = /* c */ {\n  key: string;\n};\ntype A = /* 1 */ /* 2 */ /* 3 */ /* 4 */ {\n  key: string;\n};\n",
        SourceType::ts(),
        "type M1 =\n  /* c */\n  {\n    key: string;\n  };\ntype A =\n  /* 1 */ /* 2 */ /* 3 */ /* 4 */\n  {\n    key: string;\n  };\n",
    );
}

#[test]
fn next_line_is_idempotent_with_a_block_comment_before_an_arrow_body_brace() {
    assert_next_line_is_idempotent(
        "() /**/ => //\n  () /**/ => /**/\n    () /**/ => /**/ {\n      //\n    }\n",
        SourceType::mjs(),
        "() /**/ =>\n  //\n  () /**/ =>\n  /**/\n  () /**/ => /**/\n  {\n    //\n  };\n",
    );
}

#[test]
fn next_line_is_idempotent_with_a_block_comment_before_an_arrow_body_brace_in_a_call() {
    assert_next_line_is_idempotent(
        "call(\n  () /**/ => //\n    () /**/ => /**/\n      () /**/ => /**/ {\n        //\n      }\n);\n",
        SourceType::mjs(),
        "call(() /**/ =>\n  //\n  () /**/ =>\n  /**/\n  () /**/ => /**/\n  {\n    //\n  },\n);\n",
    );
}
