use oxc_ast::ast::*;
use oxc_formatter_core::Format;
use oxc_span::GetSpan;

use crate::source_text::SourceTextExt as _;
use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        JsFormatContext, JsFormatter, JsFormatterExt as _,
        prelude::{format_with, group, soft_block_indent_with_maybe_space, soft_line_break},
        trivia::format_dangling_comments,
    },
    options::Expand,
    print::parameters::{get_this_param, should_hug_function_parameters},
    utils::statement_body::write_empty_brace_content,
    write,
};

#[derive(Clone, Copy)]
pub enum ObjectLike<'a, 'b> {
    ObjectExpression(&'b AstNode<'b, 'a, ObjectExpression<'a>>),
    TSTypeLiteral(&'b AstNode<'b, 'a, TSTypeLiteral<'a>>),
}

impl<'a> ObjectLike<'a, '_> {
    fn span(&self) -> Span {
        match self {
            ObjectLike::ObjectExpression(o) => o.span,
            ObjectLike::TSTypeLiteral(o) => o.span,
        }
    }

    fn should_hug(&self, f: &JsFormatter<'_, 'a>) -> bool {
        // Check if the object type is the type annotation of the only parameter in a function.
        // This prevents breaking object properties in cases like:
        // const fn = ({ foo }: { foo: string }) => { ... };
        matches!(self, Self::TSTypeLiteral(node) if {
            // Check if parent is TSTypeAnnotation
            matches!(node.parent(), AstNodes::TSTypeAnnotation(type_ann) if {
                match &type_ann.parent() {
                    AstNodes::FormalParameter(param) if param.initializer.is_none() => {
                        let AstNodes::FormalParameters(parameters) = &param.parent() else {
                            unreachable!()
                        };
                        let this_param = get_this_param(parameters.parent());
                        should_hug_function_parameters(parameters, this_param, false, f)

                    }
                    AstNodes::TSThisParameter(param) => {
                        matches!(param.parent(), AstNodes::Function(func) if {
                            should_hug_function_parameters(&func.params(), Some(**param), false, f)
                        })
                    },
                    _ => false,
                }
            })
        })
    }

    fn members_have_leading_newline(&self, f: &JsFormatter<'_, 'a>) -> bool {
        match self {
            Self::ObjectExpression(o) => o.as_ref().properties.first().is_some_and(|p| {
                f.source_text().contains_newline_between(o.span.start, p.span().start)
            }),
            Self::TSTypeLiteral(o) => o.as_ref().members.first().is_some_and(|p| {
                f.source_text().contains_newline_between(o.span().start, p.span().start)
            }),
        }
    }

    /// Whether moving `{` onto its own line would change what the program means.
    ///
    /// `return` / `throw` / `yield` are subject to ASI: `return\n{ a: 1 }` returns nothing.
    /// The hazard applies whenever the object is the argument's left-most token
    /// (`return { a: 1 }.a`), so start offsets are compared instead of enumerating wrappers;
    /// a parenthesized argument starts elsewhere and is correctly left alone.
    fn brace_break_is_unsafe(&self) -> bool {
        let Self::ObjectExpression(object) = self else { return false };
        let start = object.span.start;

        for ancestor in object.ancestors() {
            match ancestor {
                AstNodes::ReturnStatement(node) => {
                    return node.argument.as_ref().is_some_and(|it| it.span().start == start);
                }
                AstNodes::ThrowStatement(node) => return node.argument.span().start == start,
                AstNodes::YieldExpression(node) => {
                    return node.argument.as_ref().is_some_and(|it| it.span().start == start);
                }
                // Being the left-most token cannot hold across a statement or body boundary
                AstNodes::BlockStatement(_) | AstNodes::FunctionBody(_) | AstNodes::Program(_) => {
                    return false;
                }
                _ => {}
            }
        }

        false
    }

    /// Whether an empty object-like moves its `{` onto its own line, as prettier-next-line does:
    /// a type literal always, and an object literal only as the value of an assignment, a default,
    /// a property, or a class field (`x =` then `{`), so `f({})` and `return {` keep it attached.
    fn empty_breaks_before_brace(&self) -> bool {
        match self {
            Self::TSTypeLiteral(_) => true,
            Self::ObjectExpression(object) => matches!(
                object.parent(),
                AstNodes::AssignmentExpression(_)
                    | AstNodes::AssignmentPattern(_)
                    | AstNodes::AssignmentTargetWithDefault(_)
                    | AstNodes::AssignmentTargetPropertyIdentifier(_)
                    | AstNodes::FormalParameter(_)
                    | AstNodes::ObjectProperty(_)
                    | AstNodes::PropertyDefinition(_)
                    | AstNodes::VariableDeclarator(_)
            ),
        }
    }

    fn members_are_empty(&self) -> bool {
        match self {
            Self::ObjectExpression(o) => o.properties().is_empty(),
            Self::TSTypeLiteral(o) => o.members().is_empty(),
        }
    }

    fn write_members(&self, f: &mut JsFormatter<'_, 'a>) {
        match self {
            Self::ObjectExpression(o) => o.properties().fmt(f),
            Self::TSTypeLiteral(o) => o.members().fmt(f),
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for ObjectLike<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let members = format_with(|f| self.write_members(f));

        if self.members_are_empty() && f.options().brace_style.is_next_line() {
            // prettier-next-line always splits an empty object onto two lines
            let content = format_with(|f| write_empty_brace_content(self.span(), f));
            if self.empty_breaks_before_brace() {
                write!(f, group(&format_args!(soft_line_break(), "{", content, "}")));
            } else {
                write!(f, ["{", content, "}"]);
            }
        } else if self.members_are_empty() {
            // Soft indent so the object can stay on one line if it fits:
            // a single one-line block comment stays inline without bracket spacing
            // line comments and multiple comments still expand.
            write!(f, ["{", format_dangling_comments(self.span()).with_soft_block_indent(), "}"]);
        } else {
            let should_insert_space_around_brackets = f.options().bracket_spacing.value();
            let should_expand =
                f.options().expand == Expand::Auto && self.members_have_leading_newline(f);

            // If the object type is the type annotation of the only parameter in a function,
            // try to hug the parameter; we don't create a group and inline the contents here.
            //
            // For example:
            // ```ts
            // const fn = ({ foo }: { foo: string }) => { ... };
            //                      ^ do not break properties here
            // ```
            let should_hug = self.should_hug(f);

            let inner =
                soft_block_indent_with_maybe_space(&members, should_insert_space_around_brackets);

            if should_hug {
                write!(f, ["{", inner, "}"]);
            } else if f.options().brace_style.is_next_line() && !self.brace_break_is_unsafe() {
                // The leading break shares the members' group,
                // so `{` moves to its own line exactly when the object wraps.
                write!(
                    f,
                    [group(&format_args!(soft_line_break(), "{", inner, "}"))
                        .should_expand(should_expand)]
                );
            } else {
                write!(f, ["{", group(&inner).should_expand(should_expand), "}"]);
            }
        }
    }
}
