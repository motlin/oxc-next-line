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
fn next_line_keeps_imports_exports_and_destructuring_that_fit_inline() {
    assert_eq!(
        format_next_line_narrow(
            "import { a } from \"x\";\nexport { a };\nconst { b } = value;",
            SourceType::ts(),
        ),
        "import { a } from \"x\";\nexport { a };\nconst { b } = value;\n",
    );
}

#[test]
fn next_line_always_breaks_non_empty_objects_and_type_literals() {
    assert_next_line_is_idempotent(
        "const value = { a: 1 };\ntype Point = { x: number; y: number };\nf({ a: 1 });\nfunction r() { return { a: 1 }; }\n",
        SourceType::ts(),
        "const value =\n{\n  a: 1,\n};\ntype Point =\n{\n  x: number;\n  y: number;\n};\nf(\n  {\n    a: 1,\n  },\n);\nfunction r()\n{\n  return {\n    a: 1,\n  };\n}\n",
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

#[test]
fn next_line_breaks_a_union_that_starts_with_a_comment_and_an_object() {
    assert_next_line_is_idempotent(
        "type C = /* multi\n * line */ { a: 1 } | null;\ntype E =\n  // c\n  { a: 1 } | null;\n",
        SourceType::ts(),
        "type C =\n  | /* multi\n     * line */\n    {\n      a: 1;\n    }\n  | null;\ntype E =\n  // c\n  |\n    {\n      a: 1;\n    }\n  | null;\n",
    );
}

#[test]
fn next_line_splits_empty_function_and_arrow_bodies() {
    assert_next_line_is_idempotent(
        "const f = () => {};\nconst h = (a, b) => {};\nconst g = function () {};\nitems.forEach(function () {});\nfunction noop() {}\nasync function* gen() {}\nexport default function () {}\n",
        SourceType::mjs(),
        "const f = () =>\n{\n};\nconst h = (a, b) =>\n{\n};\nconst g = function ()\n{\n};\nitems.forEach(function () {});\nfunction noop()\n{\n}\nasync function* gen()\n{\n}\nexport default function ()\n{\n}\n",
    );
}

#[test]
fn next_line_keeps_empty_callbacks_inline() {
    assert_next_line_is_idempotent(
        "Event.debounce(call(A, B), DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD)(() => {});\nfoo(() => {});\nfoo(a, b, () => {});\nnew Promise(() => {});\nconst fnArg = useCallback(() => {}, []);\nx.then(() => {}, () => {});\n",
        SourceType::mjs(),
        "Event.debounce(call(A, B), DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD)(() => {});\nfoo(() => {});\nfoo(a, b, () => {});\nnew Promise(() => {});\nconst fnArg = useCallback(() => {}, []);\nx.then(\n  () => {},\n  () => {},\n);\n",
    );
}

#[test]
fn next_line_splits_empty_class_bodies_and_members() {
    assert_next_line_is_idempotent(
        "class Empty {}\nclass A extends B {}\nclass WithEmpty { method() {} constructor() {} get x() {} static {} }\nconst obj = { m() {} };\nexport default class {}\n",
        SourceType::mjs(),
        "class Empty\n{\n}\nclass A extends B\n{\n}\nclass WithEmpty\n{\n  method()\n  {\n  }\n  constructor()\n  {\n  }\n  get x()\n  {\n  }\n  static\n  {\n  }\n}\nconst obj =\n{\n  m()\n  {\n  },\n};\nexport default class\n{\n}\n",
    );
}

#[test]
fn next_line_splits_empty_statement_blocks() {
    assert_next_line_is_idempotent(
        "if (x) {}\nif (x) {} else {}\nfor (;;) {}\nwhile (x) {}\ndo {} while (x);\ntry { f(); } catch {}\ntry { f(); } catch (e) {} finally {}\nswitch (x) {}\n{}\nlabel: {}\n",
        SourceType::mjs(),
        "if (x)\n{\n}\nif (x)\n{\n}\nelse\n{\n}\nfor (;;)\n{\n}\nwhile (x)\n{\n}\ndo\n{\n}\nwhile (x);\ntry\n{\n  f();\n}\ncatch\n{\n}\ntry\n{\n  f();\n}\ncatch (e)\n{\n}\nfinally\n{\n}\nswitch (x)\n{\n}\n{\n}\nlabel:\n{\n}\n",
    );
}

#[test]
fn next_line_splits_empty_typescript_declarations_but_not_literals() {
    assert_next_line_is_idempotent(
        "interface I {}\nenum E {}\nnamespace N {}\ndeclare module \"m\" {}\ndeclare global {}\ntype T = {};\nconst o = {};\nlet v: {} = {};\nfunction p(a: {}) {}\nconst {} = o;\nx = {} as A;\n",
        SourceType::ts(),
        "interface I\n{\n}\nenum E\n{\n}\nnamespace N\n{\n}\ndeclare module \"m\"\n{\n}\ndeclare global\n{\n}\ntype T = {};\nconst o = {};\nlet v: {} = {};\nfunction p(a: {})\n{\n}\nconst {} = o;\nx = {} as A;\n",
    );
}

#[test]
fn next_line_keeps_empty_object_literals_and_patterns_inline() {
    assert_next_line_is_idempotent(
        "f({});\nconst arr = [{}, {}];\nconst nested = { a: {}, b: [] };\nfunction r() { return {}; }\nconst e = () => ({});\nfunction d(a = {}) {}\nfoo = bar || {};\nclass K { #p = {}; }\ntry {} catch ({}) {}\n",
        SourceType::mjs(),
        "f({});\nconst arr = [{}, {}];\nconst nested =\n{\n  a: {},\n  b: [],\n};\nfunction r()\n{\n  return {};\n}\nconst e = () => ({});\nfunction d(a = {})\n{\n}\nfoo = bar || {};\nclass K\n{\n  #p = {};\n}\ntry\n{\n}\ncatch ({})\n{\n}\n",
    );
}

#[test]
fn next_line_indents_comments_in_empty_braces() {
    assert_next_line_is_idempotent(
        "function withComment() { /* c */ }\nif (x) { /* c */ }\nclass CC { /* c */ }\nclass S { static { /* c */ } }\nlabel: { /* c */ }\nswitch (x) { /* c */ }\nfoo(() => {\n  // c\n});\n",
        SourceType::mjs(),
        "function withComment()\n{\n  /* c */\n}\nif (x)\n{\n  /* c */\n}\nclass CC\n{\n  /* c */\n}\nclass S\n{\n  static\n  {\n    /* c */\n  }\n}\nlabel:\n{\n  /* c */\n}\nswitch (x)\n{\n  /* c */\n}\nfoo(() =>\n{\n  // c\n});\n",
    );
}

/// prettier-next-line prints a blank line before the comment in an empty object or enum
/// (`{` then an empty line then `/* c */`), an artifact of its dangling-comment printing;
/// the comment is indented like any other empty-brace comment instead.
#[test]
fn next_line_indents_comments_in_empty_objects_and_type_literals() {
    assert_next_line_is_idempotent(
        "const oc = { /* c */ };\nconst ol = {\n  // line\n};\ntype TC = { /* c */ };\nenum E { /* c */ }\ninterface I {\n  // c\n}\nnamespace N { /* c */ }\n",
        SourceType::ts(),
        "const oc =\n{\n  /* c */\n};\nconst ol =\n{\n  // line\n};\ntype TC =\n{\n  /* c */\n};\nenum E\n{\n  /* c */\n}\ninterface I\n{\n  // c\n}\nnamespace N\n{\n  /* c */\n}\n",
    );
}

#[test]
fn next_line_keeps_empty_patterns_with_defaults_inline() {
    assert_next_line_is_idempotent(
        "function f({} = {}) {}\nconst [{} = x] = o;\n",
        SourceType::mjs(),
        "function f({} = {})\n{\n}\nconst [{} = x] = o;\n",
    );
}
