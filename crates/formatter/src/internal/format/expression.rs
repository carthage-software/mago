use std::borrow::Cow;
use std::collections::VecDeque;

use bumpalo::collections::Vec;
use bumpalo::vec;

use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_syntax::ast::Access;
use mago_syntax::ast::AnonymousClass;
use mago_syntax::ast::Argument;
use mago_syntax::ast::Array;
use mago_syntax::ast::ArrayAccess;
use mago_syntax::ast::ArrayAppend;
use mago_syntax::ast::ArrayElement;
use mago_syntax::ast::ArrowFunction;
use mago_syntax::ast::Assignment;
use mago_syntax::ast::AssignmentOperator;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BracedExpressionStringPart;
use mago_syntax::ast::Call;
use mago_syntax::ast::ClassConstantAccess;
use mago_syntax::ast::ClassLikeConstantSelector;
use mago_syntax::ast::ClassLikeMemberExpressionSelector;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Clone;
use mago_syntax::ast::CompositeString;
use mago_syntax::ast::Conditional;
use mago_syntax::ast::ConstantAccess;
use mago_syntax::ast::Construct;
use mago_syntax::ast::DieConstruct;
use mago_syntax::ast::DirectVariable;
use mago_syntax::ast::DocumentIndentation;
use mago_syntax::ast::DocumentKind;
use mago_syntax::ast::DocumentString;
use mago_syntax::ast::EmptyConstruct;
use mago_syntax::ast::EvalConstruct;
use mago_syntax::ast::ExitConstruct;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionPartialApplication;
use mago_syntax::ast::IncludeConstruct;
use mago_syntax::ast::IncludeOnceConstruct;
use mago_syntax::ast::IndirectVariable;
use mago_syntax::ast::Instantiation;
use mago_syntax::ast::InterpolatedString;
use mago_syntax::ast::IssetConstruct;
use mago_syntax::ast::KeyValueArrayElement;
use mago_syntax::ast::LegacyArray;
use mago_syntax::ast::List;
use mago_syntax::ast::Literal;
use mago_syntax::ast::LiteralFloat;
use mago_syntax::ast::LiteralInteger;
use mago_syntax::ast::LiteralString;
use mago_syntax::ast::LiteralStringPart;
use mago_syntax::ast::MagicConstant;
use mago_syntax::ast::Match;
use mago_syntax::ast::MatchArm;
use mago_syntax::ast::MatchDefaultArm;
use mago_syntax::ast::MatchExpressionArm;
use mago_syntax::ast::MethodPartialApplication;
use mago_syntax::ast::MissingArrayElement;
use mago_syntax::ast::NamedArgument;
use mago_syntax::ast::NamedPlaceholderArgument;
use mago_syntax::ast::NestedVariable;
use mago_syntax::ast::NullSafePropertyAccess;
use mago_syntax::ast::PartialApplication;
use mago_syntax::ast::PartialArgument;
use mago_syntax::ast::PartialArgumentList;
use mago_syntax::ast::Pipe;
use mago_syntax::ast::PlaceholderArgument;
use mago_syntax::ast::PositionalArgument;
use mago_syntax::ast::PrintConstruct;
use mago_syntax::ast::PropertyAccess;
use mago_syntax::ast::RequireConstruct;
use mago_syntax::ast::RequireOnceConstruct;
use mago_syntax::ast::ShellExecuteString;
use mago_syntax::ast::StaticMethodPartialApplication;
use mago_syntax::ast::StaticPropertyAccess;
use mago_syntax::ast::StringPart;
use mago_syntax::ast::Throw;
use mago_syntax::ast::UnaryPostfix;
use mago_syntax::ast::UnaryPostfixOperator;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
use mago_syntax::ast::ValueArrayElement;
use mago_syntax::ast::Variable;
use mago_syntax::ast::VariadicArrayElement;
use mago_syntax::ast::VariadicPlaceholderArgument;
use mago_syntax::ast::Yield;
use mago_syntax::ast::YieldFrom;
use mago_syntax::ast::YieldPair;
use mago_syntax::ast::YieldValue;

use crate::document::Align;
use crate::document::Document;
use crate::document::Line;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::Group;
use crate::internal::format::IfBreak;
use crate::internal::format::IndentIfBreak;
use crate::internal::format::Separator;
use crate::internal::format::array::ArrayLike;
use crate::internal::format::array::print_array_like;
use crate::internal::format::assignment::AssignmentLikeNode;
use crate::internal::format::assignment::print_assignment;
use crate::internal::format::assignment::print_assignment_with_alignment;
use crate::internal::format::binaryish;
use crate::internal::format::binaryish::BinaryishOperator;
use crate::internal::format::call_arguments::print_argument_list;
use crate::internal::format::call_node::CallLikeNode;
use crate::internal::format::call_node::print_call_like_node;
use crate::internal::format::class_like::print_class_like_body;
use crate::internal::format::format_token;
use crate::internal::format::member_access::collect_member_access_chain;
use crate::internal::format::member_access::print_member_access_chain;
use crate::internal::format::misc;
use crate::internal::format::misc::print_attribute_list_sequence;
use crate::internal::format::misc::print_condition;
use crate::internal::format::misc::print_modifiers;
use crate::internal::format::print_lowercase_keyword;
use crate::internal::format::return_value::format_return_value;
use crate::internal::format::string::print_string;
use crate::internal::format::string::print_uppercase_keyword;
use crate::internal::utils;
use crate::internal::utils::could_expand_value;
use crate::internal::utils::unwrap_parenthesized;
use crate::settings::BraceStyle;
use crate::wrap;

impl<'arena> Format<'arena> for Expression<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        if let Expression::Parenthesized(parenthesized) = self {
            return parenthesized.expression.format(f);
        }

        wrap!(f, self, Expression, {
            match self {
                Expression::Binary(op) => op.format(f),
                Expression::UnaryPrefix(op) => op.format(f),
                Expression::UnaryPostfix(op) => op.format(f),
                Expression::Literal(literal) => literal.format(f),
                Expression::CompositeString(c) => c.format(f),
                Expression::Assignment(op) => op.format(f),
                Expression::Conditional(op) => op.format(f),
                Expression::Array(array) => array.format(f),
                Expression::LegacyArray(legacy_array) => legacy_array.format(f),
                Expression::List(list) => list.format(f),
                Expression::ArrayAccess(a) => a.format(f),
                Expression::ArrayAppend(a) => a.format(f),
                Expression::AnonymousClass(c) => c.format(f),
                Expression::Closure(c) => c.format(f),
                Expression::ArrowFunction(a) => a.format(f),
                Expression::Variable(v) => v.format(f),
                Expression::Identifier(i) => i.format(f),
                Expression::Match(m) => m.format(f),
                Expression::Yield(y) => y.format(f),
                Expression::Construct(construct) => construct.format(f),
                Expression::Throw(t) => t.format(f),
                Expression::Clone(c) => c.format(f),
                Expression::Call(c) => {
                    if let Some(access_chain) = collect_member_access_chain(f.arena, self) {
                        if access_chain.is_eligible_for_chaining(f) {
                            print_member_access_chain(&access_chain, f)
                        } else {
                            c.format(f)
                        }
                    } else {
                        c.format(f)
                    }
                }
                Expression::Access(a) => {
                    if let Some(access_chain) = collect_member_access_chain(f.arena, self) {
                        if access_chain.is_eligible_for_chaining(f) {
                            print_member_access_chain(&access_chain, f)
                        } else {
                            a.format(f)
                        }
                    } else {
                        a.format(f)
                    }
                }
                Expression::ConstantAccess(a) => a.format(f),
                Expression::PartialApplication(p) => p.format(f),
                Expression::Parent(k) => k.format(f),
                Expression::Static(k) => k.format(f),
                Expression::Self_(k) => k.format(f),
                Expression::Instantiation(i) => i.format(f),
                Expression::MagicConstant(c) => c.format(f),
                Expression::Pipe(p) => p.format(f),
                Expression::Parenthesized(_) => unreachable!("Parenthesized expressions are handled separately"),
                _ => unreachable!("An expression variant was not handled in formatter: {self:?}"),
            }
        })
    }
}

impl<'arena> Format<'arena> for Binary<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Binary, {
            binaryish::print_binaryish_expression(f, self.lhs, BinaryishOperator::Binary(&self.operator), self.rhs)
        })
    }
}

impl<'arena> Format<'arena> for Pipe<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Pipe, {
            let has_trailing_comments = f.has_comment(self.span(), CommentFlags::Trailing);
            let mut should_break = has_trailing_comments;

            let mut callables: Vec<&'arena Expression<'arena>> = vec![in f.arena];
            let mut input: &'arena Expression<'arena> = self.input;

            callables.push(self.callable);
            while let Expression::Pipe(inner_pipe) = unwrap_parenthesized(input) {
                callables.push(inner_pipe.callable);
                input = inner_pipe.input;
            }

            // Always break if we have more than 3 callables
            should_break |= callables.len() > 3;

            callables.reverse();
            let formatted_input = input.format(f);
            let mut contents = vec![in f.arena; ];
            let mut callable_queue: VecDeque<&'arena Expression<'arena>> = callables.into_iter().collect();
            while let Some(callable) = callable_queue.pop_front() {
                contents.push(Document::Line(Line::default()));
                contents.push(Document::String("|> "));

                let callable_has_trailing_comments = f.has_comment(callable.span(), CommentFlags::Trailing);
                contents.push(callable.format(f));
                if callable_has_trailing_comments {
                    should_break = true;
                }
            }

            Document::Group(
                Group::new(vec![in f.arena; formatted_input, Document::Indent(contents)]).with_break(should_break),
            )
        })
    }
}

impl<'arena> Format<'arena> for UnaryPrefix<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, UnaryPrefix, {
            Document::Group(Group::new(vec![in f.arena; self.operator.format(f), self.operand.format(f)]))
        })
    }
}

impl<'arena> Format<'arena> for UnaryPrefixOperator<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, UnaryPrefixOperator, {
            let space_after = match self {
                UnaryPrefixOperator::ErrorControl(_) => f.settings.space_after_error_control_unary_prefix_operator,
                UnaryPrefixOperator::Reference(_) => f.settings.space_after_reference_unary_prefix_operator,
                UnaryPrefixOperator::BitwiseNot(_) => f.settings.space_after_bitwise_not_unary_prefix_operator,
                UnaryPrefixOperator::Not(_) => f.settings.space_after_logical_not_unary_prefix_operator,
                UnaryPrefixOperator::PreIncrement(_) => f.settings.space_after_increment_unary_prefix_operator,
                UnaryPrefixOperator::PreDecrement(_) => f.settings.space_after_decrement_unary_prefix_operator,
                UnaryPrefixOperator::Plus(_) | UnaryPrefixOperator::Negation(_) => {
                    f.settings.space_after_additive_unary_prefix_operator
                }
                UnaryPrefixOperator::ArrayCast(_, _)
                | UnaryPrefixOperator::BoolCast(_, _)
                | UnaryPrefixOperator::BooleanCast(_, _)
                | UnaryPrefixOperator::DoubleCast(_, _)
                | UnaryPrefixOperator::RealCast(_, _)
                | UnaryPrefixOperator::FloatCast(_, _)
                | UnaryPrefixOperator::IntCast(_, _)
                | UnaryPrefixOperator::IntegerCast(_, _)
                | UnaryPrefixOperator::ObjectCast(_, _)
                | UnaryPrefixOperator::UnsetCast(_, _)
                | UnaryPrefixOperator::StringCast(_, _)
                | UnaryPrefixOperator::BinaryCast(_, _)
                | UnaryPrefixOperator::VoidCast(_, _) => f.settings.space_after_cast_unary_prefix_operators,
            };

            let operator = Document::String(print_lowercase_keyword(f, self.as_str()));

            if space_after { Document::Array(vec![in f.arena; operator, Document::space()]) } else { operator }
        })
    }
}

impl<'arena> Format<'arena> for UnaryPostfix<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, UnaryPostfix, {
            Document::Group(Group::new(vec![in f.arena; self.operand.format(f), self.operator.format(f)]))
        })
    }
}

impl<'arena> Format<'arena> for UnaryPostfixOperator {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, UnaryPostfixOperator, { Document::String(self.as_str()) })
    }
}

impl<'arena> Format<'arena> for Literal<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Literal, {
            match self {
                Literal::String(literal) => literal.format(f),
                Literal::Integer(literal) => literal.format(f),
                Literal::Float(literal) => literal.format(f),
                Literal::True(keyword) | Literal::False(keyword) | Literal::Null(keyword) => {
                    if f.settings.uppercase_literal_keyword {
                        wrap!(f, keyword, Keyword, { Document::String(print_uppercase_keyword(f, keyword.value)) })
                    } else {
                        wrap!(f, keyword, Keyword, { Document::String(print_lowercase_keyword(f, keyword.value)) })
                    }
                }
            }
        })
    }
}

impl<'arena> Format<'arena> for LiteralString<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, LiteralString, { Document::String(print_string(f, self.kind, self.raw)) })
    }
}

impl<'arena> Format<'arena> for LiteralInteger<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, LiteralInteger, { Document::String(self.raw) })
    }
}

impl<'arena> Format<'arena> for LiteralFloat<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, LiteralFloat, { Document::String(self.raw) })
    }
}

impl<'arena> Format<'arena> for Variable<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Variable, {
            match self {
                Variable::Direct(var) => var.format(f),
                Variable::Indirect(var) => var.format(f),
                Variable::Nested(var) => var.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for IndirectVariable<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IndirectVariable, {
            Document::Group(Group::new(
                vec![in f.arena; Document::String("${"), self.expression.format(f), Document::String("}")],
            ))
        })
    }
}

impl<'arena> Format<'arena> for DirectVariable<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, DirectVariable, { Document::String(self.name) })
    }
}

impl<'arena> Format<'arena> for NestedVariable<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, NestedVariable, {
            Document::Group(Group::new(vec![in f.arena; Document::String("$"), self.variable.format(f)]))
        })
    }
}

impl<'arena> Format<'arena> for Array<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Array, { print_array_like(f, ArrayLike::Array(self)) })
    }
}

impl<'arena> Format<'arena> for LegacyArray<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, LegacyArray, { print_array_like(f, ArrayLike::LegacyArray(self)) })
    }
}

impl<'arena> Format<'arena> for List<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, List, { print_array_like(f, ArrayLike::List(self)) })
    }
}

impl<'arena> Format<'arena> for ArrayElement<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ArrayElement, {
            match self {
                ArrayElement::KeyValue(e) => e.format(f),
                ArrayElement::Value(e) => e.format(f),
                ArrayElement::Variadic(e) => e.format(f),
                ArrayElement::Missing(e) => e.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for KeyValueArrayElement<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, KeyValueArrayElement, {
            let lhs = self.key.format(f);
            let operator = Document::String("=>");

            Document::Group(Group::new(vec![in f.arena; print_assignment_with_alignment(
                f,
                AssignmentLikeNode::KeyValueArrayElement(self),
                lhs,
                operator,
                self.value,
                f.alignment_context(),
            )]))
        })
    }
}

impl<'arena> Format<'arena> for ValueArrayElement<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ValueArrayElement, { self.value.format(f) })
    }
}

impl<'arena> Format<'arena> for VariadicArrayElement<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, VariadicArrayElement, {
            Document::Array(vec![in f.arena; Document::String("..."), self.value.format(f)])
        })
    }
}

impl<'arena> Format<'arena> for MissingArrayElement {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MissingArrayElement, { Document::empty() })
    }
}

impl<'arena> Format<'arena> for Construct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Construct, {
            match self {
                Construct::Isset(c) => c.format(f),
                Construct::Empty(c) => c.format(f),
                Construct::Eval(c) => c.format(f),
                Construct::Include(c) => c.format(f),
                Construct::IncludeOnce(c) => c.format(f),
                Construct::Require(c) => c.format(f),
                Construct::RequireOnce(c) => c.format(f),
                Construct::Print(c) => c.format(f),
                Construct::Exit(c) => c.format(f),
                Construct::Die(c) => c.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for IssetConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IssetConstruct, {
            let mut contents = vec![in f.arena; self.isset.format(f), Document::String("(")];

            if !self.values.is_empty() {
                let mut values = Document::join(f.arena, self.values.iter().map(|v| v.format(f)), Separator::CommaLine);

                if f.settings.trailing_comma {
                    values.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(","))));
                }

                values.insert(0, Document::Line(Line::soft()));

                contents.push(Document::Indent(values));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(Document::String(")"));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for EmptyConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, EmptyConstruct, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.empty.format(f),
                Document::String("("),
                self.value.format(f),
                Document::String(")"),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for EvalConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, EvalConstruct, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.eval.format(f),
                Document::String("("),
                self.value.format(f),
                Document::String(")"),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for IncludeConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IncludeConstruct, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.include.format(f),
                Document::Indent(vec![in f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for IncludeOnceConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, IncludeOnceConstruct, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.include_once.format(f),
                Document::Indent(vec![in f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for RequireConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, RequireConstruct, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.require.format(f),
                Document::Indent(vec![in f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for RequireOnceConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, RequireOnceConstruct, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.require_once.format(f),
                Document::Indent(vec![in f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for PrintConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PrintConstruct, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.print.format(f),
                Document::Indent(vec![in f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for ExitConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ExitConstruct, { print_call_like_node(f, CallLikeNode::ExitConstruct(self)) })
    }
}

impl<'arena> Format<'arena> for DieConstruct<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, DieConstruct, { print_call_like_node(f, CallLikeNode::DieConstruct(self)) })
    }
}

impl<'arena> Format<'arena> for Argument<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Argument, {
            match self {
                Argument::Positional(a) => a.format(f),
                Argument::Named(a) => a.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for PositionalArgument<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PositionalArgument, {
            match self.ellipsis {
                Some(_) => Document::Group(Group::new(vec![in f.arena; Document::String("..."), self.value.format(f)])),
                None => Document::Group(Group::new(vec![in f.arena; self.value.format(f)])),
            }
        })
    }
}

impl<'arena> Format<'arena> for NamedArgument<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, NamedArgument, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.name.format(f),
                Document::String(":"),
                Document::space(),
                self.value.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for PartialArgumentList<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PartialArgumentList, {
            use crate::internal::format::call_arguments::print_partial_argument_list;
            print_partial_argument_list(f, self, false, false)
        })
    }
}

impl<'arena> Format<'arena> for PartialArgument<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PartialArgument, {
            match self {
                PartialArgument::Positional(a) => a.format(f),
                PartialArgument::Named(a) => a.format(f),
                PartialArgument::NamedPlaceholder(p) => p.format(f),
                PartialArgument::Placeholder(p) => p.format(f),
                PartialArgument::VariadicPlaceholder(p) => p.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for PlaceholderArgument {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PlaceholderArgument, { Document::String("?") })
    }
}

impl<'arena> Format<'arena> for VariadicPlaceholderArgument {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, VariadicPlaceholderArgument, { Document::String("...") })
    }
}

impl<'arena> Format<'arena> for NamedPlaceholderArgument<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, NamedPlaceholderArgument, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.name.format(f),
                Document::String(":"),
                Document::space(),
                Document::String("?"),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Assignment<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Assignment, {
            let lhs = self.lhs.format(f);

            let operator = match self.operator {
                AssignmentOperator::Assign(_) => Document::String("="),
                AssignmentOperator::Addition(_) => Document::String("+="),
                AssignmentOperator::Subtraction(_) => Document::String("-="),
                AssignmentOperator::Multiplication(_) => Document::String("*="),
                AssignmentOperator::Division(_) => Document::String("/="),
                AssignmentOperator::Modulo(_) => Document::String("%="),
                AssignmentOperator::Exponentiation(_) => Document::String("**="),
                AssignmentOperator::Concat(_) => Document::String(".="),
                AssignmentOperator::BitwiseAnd(_) => Document::String("&="),
                AssignmentOperator::BitwiseOr(_) => Document::String("|="),
                AssignmentOperator::BitwiseXor(_) => Document::String("^="),
                AssignmentOperator::LeftShift(_) => Document::String("<<="),
                AssignmentOperator::RightShift(_) => Document::String(">>="),
                AssignmentOperator::Coalesce(_) => Document::String("??="),
            };

            print_assignment_with_alignment(
                f,
                AssignmentLikeNode::AssignmentOperation(self),
                lhs,
                operator,
                self.rhs,
                f.alignment_context(),
            )
        })
    }
}

impl<'arena> Format<'arena> for ArrowFunction<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ArrowFunction, {
            let mut contents = vec![in f.arena];
            if let Some(attributes) = print_attribute_list_sequence(f, &self.attribute_lists) {
                contents.push(attributes);
                contents.push(Document::Line(Line::default()));
            }

            if let Some(s) = &self.r#static {
                contents.push(s.format(f));
                contents.push(Document::space());
            }

            contents.push(self.r#fn.format(f));
            if f.settings.space_before_arrow_function_parameter_list_parenthesis {
                contents.push(Document::space());
            }

            if self.ampersand.is_some() {
                contents.push(Document::String("&"));
            }

            contents.push(self.parameter_list.format(f));
            if let Some(h) = &self.return_type_hint {
                contents.push(h.format(f));
            }

            contents.push(Document::String(" => "));
            contents.push(format_return_value(f, self.expression));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for ClassLikeMemberSelector<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ClassLikeMemberSelector, {
            match self {
                ClassLikeMemberSelector::Identifier(s) => s.format(f),
                ClassLikeMemberSelector::Variable(s) => s.format(f),
                ClassLikeMemberSelector::Expression(s) => s.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for ClassLikeMemberExpressionSelector<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ClassLikeMemberExpressionSelector, {
            Document::Group(Group::new(
                vec![in f.arena; Document::String("{"), self.expression.format(f), Document::String("}")],
            ))
        })
    }
}

impl<'arena> Format<'arena> for ClassLikeConstantSelector<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ClassLikeConstantSelector, {
            match self {
                ClassLikeConstantSelector::Identifier(s) => s.format(f),
                ClassLikeConstantSelector::Expression(s) => s.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for ConstantAccess<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ConstantAccess, { self.name.format(f) })
    }
}

impl<'arena> Format<'arena> for Access<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Access, {
            match self {
                Access::Property(a) => a.format(f),
                Access::NullSafeProperty(a) => a.format(f),
                Access::StaticProperty(a) => a.format(f),
                Access::ClassConstant(a) => a.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for PropertyAccess<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PropertyAccess, {
            Document::Group(Group::new(
                vec![in f.arena; self.object.format(f), Document::String("->"), self.property.format(f)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for NullSafePropertyAccess<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, NullSafePropertyAccess, {
            Document::Group(Group::new(
                vec![in f.arena; self.object.format(f), Document::String("?->"), self.property.format(f)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for StaticPropertyAccess<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, StaticPropertyAccess, {
            Document::Group(Group::new(
                vec![in f.arena; self.class.format(f), Document::String("::"), self.property.format(f)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for ClassConstantAccess<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ClassConstantAccess, {
            Document::Group(Group::new(
                vec![in f.arena; self.class.format(f), Document::String("::"), self.constant.format(f)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for Call<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Call, { print_call_like_node(f, CallLikeNode::Call(self)) })
    }
}

impl<'arena> Format<'arena> for Throw<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Throw, {
            Document::Group(Group::new(
                vec![in f.arena; self.throw.format(f), Document::space(), self.exception.format(f)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for Instantiation<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Instantiation, { print_call_like_node(f, CallLikeNode::Instantiation(self)) })
    }
}

impl<'arena> Format<'arena> for ArrayAccess<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ArrayAccess, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.array.format(f),
                Document::String("["),
                self.index.format(f),
                Document::String("]"),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for ArrayAppend<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ArrayAppend, {
            Document::Group(Group::new(vec![in f.arena; self.array.format(f), Document::String("[]")]))
        })
    }
}

impl<'arena> Format<'arena> for MatchArm<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MatchArm, {
            match self {
                MatchArm::Expression(a) => a.format(f),
                MatchArm::Default(a) => a.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for MatchDefaultArm<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MatchDefaultArm, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.default.format(f),
                format_token(f, self.arrow, " => "),
                self.expression.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for MatchExpressionArm<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MatchExpressionArm, {
            let len = self.conditions.len();

            let must_break = self
                .conditions
                .iter()
                .take(len.saturating_sub(1))
                .any(|condition| f.has_comment(condition.span(), CommentFlags::Trailing | CommentFlags::Line));

            let mut contents = vec![in f.arena];
            for (i, condition) in self.conditions.iter().enumerate() {
                contents.push(condition.format(f));
                if i != (len - 1) {
                    contents.push(Document::String(","));
                    contents.push(if must_break {
                        Document::Line(Line::hard())
                    } else {
                        Document::Line(Line::default())
                    });
                } else if f.settings.trailing_comma && i > 0 {
                    contents.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(","))));
                }
            }

            let group_id = f.next_id();
            contents.push(Document::IndentIfBreak(IndentIfBreak::new(
                group_id,
                vec![
                    in f.arena;
                    if must_break { Document::Line(Line::hard()) } else { Document::Line(Line::default()) },
                    format_token(f, self.arrow, "=> "),
                ],
            )));

            Document::Group(
                Group::new(vec![
                    in f.arena;
                    Document::Group(Group::new(contents).with_break(must_break)),
                    self.expression.format(f),
                ])
                .with_id(group_id)
                .with_break(must_break),
            )
        })
    }
}

impl<'arena> Format<'arena> for Match<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Match, {
            let mut contents = vec![
                in f.arena;
                self.r#match.format(f),
                print_condition(
                    f,
                    self.left_parenthesis,
                    self.expression,
                    self.right_parenthesis,
                ),
            ];

            match f.settings.control_brace_style {
                BraceStyle::SameLine => {
                    contents.push(Document::space());
                }
                BraceStyle::NextLine => {
                    contents.push(Document::Line(Line::default()));
                }
                BraceStyle::AlwaysNextLine => {
                    contents.push(Document::Line(Line::hard()));
                }
            }

            contents.push(format_token(f, self.left_brace, "{"));

            let should_break = self.arms.len() > 1
                || self.arms.iter().any(|arm| {
                    misc::has_new_line_in_range(
                        f.source_text,
                        arm.start_position().offset(),
                        arm.end_position().offset(),
                    )
                });

            if !self.arms.is_empty() {
                let mut arms_document = Document::join(
                    f.arena,
                    self.arms.iter().map(|arm| arm.format(f)),
                    if should_break { Separator::CommaHardLine } else { Separator::CommaLine },
                );

                if f.settings.trailing_comma {
                    if should_break {
                        arms_document.push(Document::String(","));
                    } else {
                        arms_document.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(","))));
                    }
                }

                contents.push(Document::Indent(vec![
                    in f.arena;
                    if should_break { Document::Line(Line::hard()) } else { Document::Line(Line::default()) },
                    Document::Array(arms_document),
                ]));
            }

            if let Some(comments) = f.print_dangling_comments(self.left_brace.join(self.right_brace), true) {
                contents.push(comments);
            } else {
                contents.push(if should_break {
                    Document::Line(Line::hard())
                } else {
                    Document::Line(Line::default())
                });
            }

            contents.push(format_token(f, self.right_brace, "}"));

            Document::Group(Group::new(contents).with_break(should_break))
        })
    }
}

impl<'arena> Format<'arena> for Conditional<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Conditional, {
            let must_break = f.settings.preserve_breaking_conditional_expression && {
                misc::has_new_line_in_range(f.source_text, self.condition.span().end.offset, self.colon.start.offset)
                    || self.then.as_ref().is_some_and(|t| {
                        misc::has_new_line_in_range(
                            f.source_text,
                            self.question_mark.start.offset,
                            t.span().start.offset,
                        )
                    })
            };

            match &self.then {
                Some(then) => {
                    let inline_colon = !misc::has_new_line_in_range(
                        f.source_text,
                        then.span().end.offset,
                        self.r#else.span().start.offset,
                    ) && could_expand_value(f, then, false);

                    let conditional_id = f.next_id();
                    let then_id = f.next_id();

                    let break_group = must_break
                        && matches!(unwrap_parenthesized(self.condition), Expression::Binary(Binary { lhs, rhs, .. }) if lhs.is_binary() || rhs.is_binary());

                    Document::Group(
                        Group::new(vec![
                            in f.arena;
                            self.condition.format(f),
                            Document::Indent(vec![
                                in f.arena;
                                Document::Line(if must_break { Line::hard() } else { Line::default() }),
                                format_token(f, self.question_mark, "? "),
                                Document::Group(Group::new(vec![in f.arena; then.format(f)]).with_id(then_id)),
                                {
                                    if inline_colon {
                                        if must_break {
                                            Document::space()
                                        } else {
                                            Document::IfBreak(
                                                IfBreak::new(f.arena, Document::space(), {
                                                    Document::IfBreak(
                                                        IfBreak::new(f.arena, Document::Line(Line::hard()), Document::space())
                                                            .with_id(conditional_id),
                                                    )
                                                })
                                                .with_id(then_id),
                                            )
                                        }
                                    } else {
                                        Document::Line(if must_break { Line::hard() } else { Line::default() })
                                    }
                                },
                                format_token(f, self.colon, ": "),
                                self.r#else.format(f),
                            ]),
                        ])
                        .with_break(break_group)
                        .with_id(conditional_id),
                    )
                }
                None => binaryish::print_binaryish_expression(
                    f,
                    self.condition,
                    BinaryishOperator::Elvis(self.question_mark.join(self.colon)),
                    self.r#else,
                ),
            }
        })
    }
}

impl<'arena> Format<'arena> for CompositeString<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, CompositeString, {
            match self {
                CompositeString::ShellExecute(s) => s.format(f),
                CompositeString::Interpolated(s) => s.format(f),
                CompositeString::Document(s) => s.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for DocumentString<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, DocumentString, {
            let mut contents = vec![in f.arena; Document::LineSuffixBoundary, Document::String("<<<")];
            match self.kind {
                DocumentKind::Heredoc => {
                    contents.push(Document::String(self.label));
                }
                DocumentKind::Nowdoc => {
                    contents.push(Document::String("'"));
                    contents.push(Document::String(self.label));
                    contents.push(Document::String("'"));
                }
            }

            let indent = match self.indentation {
                DocumentIndentation::None => 0,
                DocumentIndentation::Whitespace(n) => n,
                DocumentIndentation::Tab(n) => n,
                DocumentIndentation::Mixed(t, w) => t + w,
            };

            let mut inner = vec![in f.arena; Document::Line(Line::hard())];

            // Track the indentation from the last line of the previous literal part
            let mut last_part_indentation = Cow::Borrowed("");

            for part in &self.parts {
                let formatted = if let StringPart::Literal(l) = part {
                    let content = l.value;
                    let mut part_contents = vec![in f.arena;];
                    let own_line = f.has_newline(l.span.start.offset, true);
                    let lines = f.split_lines(content);

                    for line in &lines {
                        let mut line = *line;
                        if own_line {
                            line = FormatterState::skip_leading_whitespace_up_to(line, indent);
                        }

                        let mut line_content = vec![in f.arena; Document::String(line)];
                        if !line.is_empty() {
                            line_content.push(Document::DoNotTrim);
                        }

                        part_contents.push(Document::Array(line_content));
                    }

                    part_contents = Document::join(f.arena, part_contents, Separator::HardLine);

                    // if ends with a newline, add a newline
                    if content.ends_with('\n') {
                        part_contents.push(Document::Line(Line::hard()));
                    }

                    // Calculate indentation from the last line of this literal part
                    // We need to use the stripped line (after removing heredoc indent)
                    if let Some(last_line) = lines.last() {
                        let stripped_line = if own_line {
                            FormatterState::skip_leading_whitespace_up_to(last_line, indent)
                        } else {
                            *last_line
                        };

                        let mut tabs = 0;
                        let mut spaces = 0;
                        for ch in stripped_line.chars() {
                            match ch {
                                '\t' => tabs += 1,
                                ' ' => spaces += 1,
                                _ => break,
                            }
                        }

                        if tabs > 0 || spaces > 0 {
                            last_part_indentation = if tabs > 0 {
                                Cow::Owned("\t".repeat(tabs) + &" ".repeat(spaces))
                            } else {
                                Cow::Owned(" ".repeat(spaces))
                            };
                        } else {
                            last_part_indentation = Cow::Borrowed("");
                        }
                    }

                    Document::Array(part_contents)
                } else {
                    let (base_alignment, adjusted_last_part) = if f.settings.indent_heredoc {
                        let scope = if f.settings.use_tabs {
                            Cow::Borrowed("\t")
                        } else {
                            Cow::Owned(" ".repeat(f.settings.tab_width))
                        };

                        let adjusted = if !last_part_indentation.is_empty() {
                            Cow::Owned(format!("{scope}{last_part_indentation}"))
                        } else {
                            Cow::Borrowed("")
                        };

                        (scope, adjusted)
                    } else {
                        let base = match self.indentation {
                            DocumentIndentation::None => Cow::Borrowed(""),
                            DocumentIndentation::Whitespace(n) => Cow::Owned(" ".repeat(n)),
                            DocumentIndentation::Tab(n) => Cow::Owned("\t".repeat(n)),
                            DocumentIndentation::Mixed(t, w) => Cow::Owned("\t".repeat(t) + &" ".repeat(w)),
                        };

                        (base, last_part_indentation.clone())
                    };

                    let combined_alignment = if !base_alignment.is_empty() || !adjusted_last_part.is_empty() {
                        Cow::Owned(format!("{base_alignment}{adjusted_last_part}"))
                    } else {
                        Cow::Borrowed("")
                    };

                    Document::Align(Align {
                        alignment: f.as_str(&combined_alignment),
                        contents: vec![
                            in f.arena;
                            part.format(f)
                        ],
                    })
                };

                inner.push(formatted);
            }

            inner.push(Document::String(self.label));

            if f.settings.indent_heredoc {
                contents.push(Document::Indent(inner));
            } else {
                contents.extend(inner);
            }

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena> Format<'arena> for InterpolatedString<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, InterpolatedString, {
            let mut parts = vec![in f.arena; Document::String("\"")];
            let mut last_part_indentation = Cow::Borrowed("");

            for part in &self.parts {
                let formatted = match part {
                    StringPart::Literal(l) => {
                        let lines = f.split_lines(l.value);
                        if let Some(last_line) = lines.last() {
                            let mut tabs = 0;
                            let mut spaces = 0;
                            for ch in last_line.chars() {
                                match ch {
                                    '\t' => tabs += 1,
                                    ' ' => spaces += 1,
                                    _ => break,
                                }
                            }
                            if tabs > 0 || spaces > 0 {
                                last_part_indentation = if tabs > 0 {
                                    Cow::Owned("\t".repeat(tabs) + &" ".repeat(spaces))
                                } else {
                                    Cow::Owned(" ".repeat(spaces))
                                };
                            } else {
                                last_part_indentation = Cow::Borrowed("");
                            }
                        }
                        part.format(f)
                    }
                    _ => {
                        if last_part_indentation.is_empty() {
                            part.format(f)
                        } else {
                            Document::Align(Align {
                                alignment: f.as_str(&last_part_indentation),
                                contents: vec![in f.arena; part.format(f)],
                            })
                        }
                    }
                };
                parts.push(formatted);
            }

            parts.push(Document::String("\""));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena> Format<'arena> for ShellExecuteString<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, ShellExecuteString, {
            let mut parts = vec![in f.arena; Document::String("`")];
            let mut last_part_indentation = Cow::Borrowed("");

            for part in &self.parts {
                let formatted = match part {
                    StringPart::Literal(l) => {
                        let lines = f.split_lines(l.value);
                        if let Some(last_line) = lines.last() {
                            let mut tabs = 0;
                            let mut spaces = 0;
                            for ch in last_line.chars() {
                                match ch {
                                    '\t' => tabs += 1,
                                    ' ' => spaces += 1,
                                    _ => break,
                                }
                            }
                            if tabs > 0 || spaces > 0 {
                                last_part_indentation = if tabs > 0 {
                                    Cow::Owned("\t".repeat(tabs) + &" ".repeat(spaces))
                                } else {
                                    Cow::Owned(" ".repeat(spaces))
                                };
                            } else {
                                last_part_indentation = Cow::Borrowed("");
                            }
                        }
                        part.format(f)
                    }
                    StringPart::BracedExpression(_) => {
                        if last_part_indentation.is_empty() {
                            part.format(f)
                        } else {
                            Document::Align(Align {
                                alignment: f.as_str(&last_part_indentation),
                                contents: vec![in f.arena; part.format(f)],
                            })
                        }
                    }
                    _ => part.format(f),
                };
                parts.push(formatted);
            }

            parts.push(Document::String("`"));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena> Format<'arena> for StringPart<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, StringPart, {
            match self {
                StringPart::Literal(s) => s.format(f),
                StringPart::Expression(s) => s.format(f),
                StringPart::BracedExpression(s) => s.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for LiteralStringPart<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, LiteralStringPart, {
            utils::replace_end_of_line(f, Document::String(self.value), Separator::LiteralLine, false)
        })
    }
}

impl<'arena> Format<'arena> for BracedExpressionStringPart<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, BracedExpressionStringPart, {
            Document::Group(Group::new(
                vec![in f.arena; Document::String("{"), self.expression.format(f), Document::String("}")],
            ))
        })
    }
}

impl<'arena> Format<'arena> for Yield<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Yield, {
            match self {
                Yield::Value(y) => y.format(f),
                Yield::Pair(y) => y.format(f),
                Yield::From(y) => y.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for YieldValue<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, YieldValue, {
            match &self.value {
                Some(v) => Document::Group(Group::new(
                    vec![in f.arena; self.r#yield.format(f), Document::space(), v.format(f)],
                )),
                None => self.r#yield.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for YieldPair<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, YieldPair, {
            let group_id = f.next_id();

            let yield_keyword = self.r#yield.format(f);
            let key_document = self.key.format(f);

            Document::Group(
                Group::new(vec![
                    in f.arena;
                    yield_keyword,
                    Document::space(),
                    print_assignment(
                        f,
                        AssignmentLikeNode::YieldPair(self),
                        key_document,
                        Document::String("=>"),
                        self.value,
                    ),
                ])
                .with_id(group_id),
            )
        })
    }
}

impl<'arena> Format<'arena> for YieldFrom<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, YieldFrom, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.r#yield.format(f),
                Document::space(),
                self.from.format(f),
                Document::space(),
                self.iterator.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for Clone<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, Clone, {
            Document::Group(Group::new(
                vec![in f.arena; self.clone.format(f), Document::space(), self.object.format(f)],
            ))
        })
    }
}

impl<'arena> Format<'arena> for MagicConstant<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MagicConstant, {
            match &self {
                MagicConstant::Line(i) => i.format(f),
                MagicConstant::File(i) => i.format(f),
                MagicConstant::Directory(i) => i.format(f),
                MagicConstant::Trait(i) => i.format(f),
                MagicConstant::Method(i) => i.format(f),
                MagicConstant::Function(i) => i.format(f),
                MagicConstant::Property(i) => i.format(f),
                MagicConstant::Namespace(i) => i.format(f),
                MagicConstant::Class(i) => i.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for PartialApplication<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, PartialApplication, {
            match &self {
                PartialApplication::Function(p) => p.format(f),
                PartialApplication::Method(p) => p.format(f),
                PartialApplication::StaticMethod(p) => p.format(f),
            }
        })
    }
}

impl<'arena> Format<'arena> for FunctionPartialApplication<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, FunctionPartialApplication, {
            Document::Group(Group::new(vec![in f.arena; self.function.format(f), self.argument_list.format(f)]))
        })
    }
}

impl<'arena> Format<'arena> for MethodPartialApplication<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, MethodPartialApplication, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.object.format(f),
                Document::String("->"),
                self.method.format(f),
                self.argument_list.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for StaticMethodPartialApplication<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, StaticMethodPartialApplication, {
            Document::Group(Group::new(vec![
                in f.arena;
                self.class.format(f),
                Document::String("::"),
                self.method.format(f),
                self.argument_list.format(f),
            ]))
        })
    }
}

impl<'arena> Format<'arena> for AnonymousClass<'arena> {
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena>) -> Document<'arena> {
        wrap!(f, self, AnonymousClass, {
            let initialization = {
                let mut contents = vec![in f.arena; self.new.format(f)];
                if let Some(attributes) = misc::print_attribute_list_sequence(f, &self.attribute_lists) {
                    contents.push(Document::Line(Line::default()));
                    contents.push(attributes);
                    contents.push(Document::Line(Line::hard()));
                } else {
                    contents.push(Document::space());
                }

                Document::Group(Group::new(contents))
            };

            let mut signature = print_modifiers(f, &self.modifiers);
            if !signature.is_empty() {
                signature.push(Document::space());
            }

            signature.push(self.class.format(f));
            if let Some(argument_list) = &self.argument_list {
                signature.push(print_argument_list(f, argument_list, false, false));
            }

            if let Some(extends) = &self.extends {
                signature.push(Document::space());
                signature.push(extends.format(f));
            }

            if let Some(implements) = &self.implements {
                signature.push(Document::space());
                signature.push(implements.format(f));
            }

            let signature_id = f.next_id();
            let signature = Document::Group(Group::new(signature).with_id(signature_id));

            Document::Group(Group::new(vec![
                in f.arena;
                initialization,
                signature,
                print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, Some(signature_id)),
            ]))
        })
    }
}
