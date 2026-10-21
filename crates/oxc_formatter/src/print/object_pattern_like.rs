use oxc_ast::ast::*;
use oxc_formatter_core::Format;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        JsFormatContext, JsFormatter,
        prelude::{format_with, group, soft_block_indent_with_maybe_space, soft_line_break},
        trivia::format_dangling_comments,
    },
    utils::{object::should_preserve_quote, statement_body::write_empty_brace_content},
    write,
};

use super::{
    assignment_pattern_property_list::AssignmentTargetPropertyList,
    binding_property_list::BindingPropertyList,
};

pub enum ObjectPatternLike<'a, 'b> {
    ObjectPattern(&'b AstNode<'b, 'a, ObjectPattern<'a>>),
    ObjectAssignmentTarget(&'b AstNode<'b, 'a, ObjectAssignmentTarget<'a>>),
}

impl GetSpan for ObjectPatternLike<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::ObjectPattern(node) => node.span,
            Self::ObjectAssignmentTarget(node) => node.span,
        }
    }
}

impl<'a> ObjectPatternLike<'a, '_> {
    fn is_empty(&self) -> bool {
        match self {
            Self::ObjectPattern(o) => o.is_empty(),
            Self::ObjectAssignmentTarget(o) => o.is_empty(),
        }
    }

    /// A parameter pattern never groups: `FormalParameter` groups the pattern with its type annotation (as Prettier's `printObject` does),
    /// so a source-broken type literal must break the pattern too (`{ log, logger }: {\n ... }`); a group here would cut that.
    fn is_inline(&self, _f: &JsFormatter<'_, 'a>) -> bool {
        match self {
            Self::ObjectPattern(node) => match node.parent() {
                AstNodes::FormalParameter(_) => true,
                AstNodes::AssignmentPattern(_) => {
                    matches!(node.grand_parent(), AstNodes::FormalParameter(_))
                }
                _ => false,
            },
            Self::ObjectAssignmentTarget(_) => false,
        }
    }

    fn is_catch_parameter(&self) -> bool {
        matches!(self, Self::ObjectPattern(node) if matches!(node.parent(), AstNodes::CatchParameter(_)))
    }

    /// Based on <https://github.com/prettier/prettier/blob/2d6877fcd1b78f2624e22d0ddb17a895ab12ac07/src/language-js/print/object.js#L77-L103>
    fn should_break_properties(&self) -> bool {
        match self {
            Self::ObjectPattern(node) => {
                let parent_is_parameter_or_assignment_pattern = matches!(
                    node.parent(),
                    AstNodes::CatchParameter(_)
                        | AstNodes::FormalParameter(_)
                        | AstNodes::AssignmentPattern(_)
                );

                if parent_is_parameter_or_assignment_pattern {
                    return false;
                }

                node.properties.iter().any(|property| {
                    matches!(
                        &property.value,
                        BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_)
                    )
                })
            }
            Self::ObjectAssignmentTarget(node) => {
                node.properties.iter().any(|property| match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(_) => false,
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                        matches!(
                            &prop.binding,
                            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(_)
                                | AssignmentTargetMaybeDefault::ArrayAssignmentTarget(_)
                        )
                    }
                })
            }
        }
    }

    fn is_in_assignment_like(&self) -> bool {
        match self {
            Self::ObjectPattern(node) => matches!(node.parent(), AstNodes::VariableDeclarator(_)),
            Self::ObjectAssignmentTarget(node) => matches!(
                node.parent(),
                AstNodes::AssignmentExpression(_) | AstNodes::VariableDeclarator(_)
            ),
        }
    }

    fn layout(&self, f: &JsFormatter<'_, 'a>) -> ObjectPatternLayout {
        if self.is_empty() {
            return ObjectPatternLayout::Empty;
        }

        if self.is_inline(f) {
            return ObjectPatternLayout::Inline;
        }

        let break_properties = self.should_break_properties();

        if break_properties {
            ObjectPatternLayout::Group { expand: true }
        } else if self.is_in_assignment_like() {
            if f.options().brace_style.is_next_line() {
                ObjectPatternLayout::AssignmentLeft
            } else {
                ObjectPatternLayout::Inline
            }
        } else {
            ObjectPatternLayout::Group { expand: false }
        }
    }

    fn write_properties(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_consistent = f.options().quote_properties.is_consistent();
        if is_consistent {
            let quote_needed = match self {
                Self::ObjectPattern(o) => o
                    .properties
                    .iter()
                    .any(|property| should_preserve_quote(&property.key, property.computed, f)),
                Self::ObjectAssignmentTarget(o) => o.properties.iter().any(|property| {
                    matches!(property, AssignmentTargetProperty::AssignmentTargetPropertyProperty(property)
                        if should_preserve_quote(&property.name, property.computed, f))
                }),
            };
            f.context_mut().push_quote_needed(quote_needed);
        }

        match self {
            Self::ObjectPattern(o) => BindingPropertyList::new(o.properties(), o.rest()).fmt(f),
            Self::ObjectAssignmentTarget(o) => {
                AssignmentTargetPropertyList::new(o.properties(), o.rest()).fmt(f);
            }
        }

        if is_consistent {
            f.context_mut().pop_quote_needed();
        }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for ObjectPatternLike<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        let should_insert_space_around_brackets = f.options().bracket_spacing.value();
        let format_properties = format_with(|f| {
            write!(
                f,
                soft_block_indent_with_maybe_space(
                    &format_with(|f| self.write_properties(f)),
                    should_insert_space_around_brackets
                )
            );
        });

        match self.layout(f) {
            ObjectPatternLayout::Empty if f.options().brace_style.is_next_line() => {
                // prettier-next-line always splits an empty pattern onto two lines,
                // breaking before the `{` unless the pattern is a parameter: `catch ({` then `})`.
                let content = format_with(|f| write_empty_brace_content(self.span(), f));
                if self.is_inline(f) || self.is_catch_parameter() {
                    write!(f, ["{", content, "}"]);
                } else {
                    write!(f, group(&format_args!(soft_line_break(), "{", content, "}")));
                }
            }
            ObjectPatternLayout::Empty => {
                write!(
                    f,
                    ["{", format_dangling_comments(self.span()).with_soft_block_indent(), "}"]
                );
            }
            ObjectPatternLayout::Inline => {
                write!(f, ["{", format_properties, "}"]);
            }
            ObjectPatternLayout::AssignmentLeft => {
                write!(f, [soft_line_break(), "{", format_properties, "}"]);
            }
            ObjectPatternLayout::Group { expand } if f.options().brace_style.is_next_line() => {
                // The leading break shares the properties' group,
                // so `{` moves to its own line exactly when the pattern wraps.
                write!(
                    f,
                    [group(&format_args!(soft_line_break(), "{", format_properties, "}"))
                        .should_expand(expand)]
                );
            }
            ObjectPatternLayout::Group { expand } => {
                write!(f, ["{", group(&format_properties).should_expand(expand), "}"]);
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum ObjectPatternLayout {
    /// Wrap the properties in a group with `should_expand` equal to `expand`.
    ///
    /// This is the default layout when no other special case applies.
    Group { expand: bool },

    /// Layout for a pattern without any properties.
    Empty,

    /// Don't wrap the properties in a group and instead "inline" them in the parent.
    ///
    /// Desired if the pattern is a parameter of a function that `should hug` OR
    /// if the pattern is the left side of an assignment.
    Inline,

    /// [`ObjectPatternLayout::Inline`] for the left side of an assignment under next-line braces:
    /// the `{` also moves to its own line when the assignment's group breaks the properties.
    AssignmentLeft,
}
