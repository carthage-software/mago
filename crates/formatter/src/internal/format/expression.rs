use mago_allocator::Arena;
use mago_allocator::vec::Vec as BumpVec;
use mago_allocator::vec_in;
use std::borrow::Cow;
use std::collections::VecDeque;

use mago_allocator::vec::Vec;

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
use mago_syntax::ast::Node;
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
use crate::document::BreakMode;
use crate::document::Document;
use crate::document::Line;
use crate::document::group::GroupIdentifier;
use crate::internal::FormatterState;
use crate::internal::comment::CommentFlags;
use crate::internal::format::Format;
use crate::internal::format::Group;
use crate::internal::format::IfBreak;
use crate::internal::format::IndentIfBreak;
use crate::internal::format::Separator;
use crate::internal::format::alignment::AlignmentRun;
use crate::internal::format::alignment::AlignmentWidths;
use crate::internal::format::alignment::has_comment_between;
use crate::internal::format::array::ArrayLike;
use crate::internal::format::array::print_array_like;
use crate::internal::format::assignment::AssignmentLikeNode;
use crate::internal::format::assignment::print_assignment;
use crate::internal::format::assignment::print_assignment_with_alignment;
use crate::internal::format::binaryish;
use crate::internal::format::binaryish::BinaryishOperator;
use crate::internal::format::call_arguments::print_argument_list;
use crate::internal::format::call_arguments::print_partial_argument_list;
use crate::internal::format::call_node::CallLikeNode;
use crate::internal::format::call_node::print_call_like_node;
use crate::internal::format::class_like::print_class_like_body;
use crate::internal::format::format_token;
use crate::internal::format::format_token_with_only_leading_comments;
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
use crate::internal::utils::get_expression_width;
use crate::internal::utils::unwrap_parenthesized;
use crate::settings::BraceStyle;
use crate::wrap;

impl<'arena, A> Format<'arena, A> for Expression<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
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
                Expression::Error(_) => Document::empty(),
                #[allow(clippy::unreachable)]
                Expression::Parenthesized(_) => unreachable!("Parenthesized expressions are handled separately"),
                #[allow(clippy::unreachable)]
                _ => unreachable!("An expression variant was not handled in formatter: {self:?}"),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for Binary<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Binary, {
            binaryish::print_binaryish_expression(f, self.lhs, BinaryishOperator::Binary(&self.operator), self.rhs)
        })
    }
}

impl<'arena, A> Format<'arena, A> for Pipe<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Pipe, {
            let has_trailing_comments = f.has_comment(self.span(), CommentFlags::TRAILING);
            let mut should_break = has_trailing_comments;

            let mut callables: Vec<'arena, &'arena Expression<'arena>, A> = vec_in![f.arena];
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
            let mut contents = vec_in![f.arena; ];
            let mut callable_queue: VecDeque<&'arena Expression<'arena>> = callables.into_iter().collect();
            while let Some(callable) = callable_queue.pop_front() {
                contents.push(Document::Line(Line::default()));
                contents.push(Document::String(b"|> "));

                let callable_has_trailing_comments = f.has_comment(callable.span(), CommentFlags::TRAILING);
                contents.push(callable.format(f));
                if callable_has_trailing_comments {
                    should_break = true;
                }
            }

            Document::Group(
                Group::new(vec_in![f.arena; formatted_input, Document::Indent(contents)])
                    .with_break_mode(if should_break { BreakMode::Force } else { BreakMode::Auto }),
            )
        })
    }
}

impl<'arena, A> Format<'arena, A> for UnaryPrefix<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, UnaryPrefix, {
            let operator = self.operator.format(f);
            let operand_trailing_comments = if let Expression::Parenthesized(p) = self.operand {
                f.print_trailing_comments_between_nodes(p.left_parenthesis, p.expression.span())
            } else {
                None
            };

            match operand_trailing_comments {
                Some(operand_trailing_comments) => Document::Group(Group::new(
                    vec_in![f.arena; operator, operand_trailing_comments, self.operand.format(f)],
                )),
                None => Document::Group(Group::new(vec_in![f.arena; operator, self.operand.format(f)])),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for UnaryPrefixOperator<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
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

            let operator = Document::String(print_lowercase_keyword(f, self.as_bytes()));

            if space_after { Document::Array(vec_in![f.arena; operator, Document::space()]) } else { operator }
        })
    }
}

impl<'arena, A> Format<'arena, A> for UnaryPostfix<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, UnaryPostfix, {
            Document::Group(Group::new(vec_in![f.arena; self.operand.format(f), self.operator.format(f)]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for UnaryPostfixOperator
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, UnaryPostfixOperator, { Document::String(self.as_str().as_bytes()) })
    }
}

impl<'arena, A> Format<'arena, A> for Literal<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
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

impl<'arena, A> Format<'arena, A> for LiteralString<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, LiteralString, { Document::String(print_string(f, self.kind, self.raw)) })
    }
}

impl<'arena, A> Format<'arena, A> for LiteralInteger<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, LiteralInteger, { Document::String(self.raw) })
    }
}

impl<'arena, A> Format<'arena, A> for LiteralFloat<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, LiteralFloat, { Document::String(self.raw) })
    }
}

impl<'arena, A> Format<'arena, A> for Variable<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Variable, {
            match self {
                Variable::Direct(var) => var.format(f),
                Variable::Indirect(var) => var.format(f),
                Variable::Nested(var) => var.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for IndirectVariable<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IndirectVariable, {
            Document::Group(Group::new(
                vec_in![f.arena; Document::String(b"${"), self.expression.format(f), Document::String(b"}")],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for DirectVariable<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, DirectVariable, { Document::String(self.name) })
    }
}

impl<'arena, A> Format<'arena, A> for NestedVariable<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, NestedVariable, {
            Document::Group(Group::new(vec_in![f.arena; Document::String(b"$"), self.variable.format(f)]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for Array<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Array, { print_array_like(f, ArrayLike::Array(self)) })
    }
}

impl<'arena, A> Format<'arena, A> for LegacyArray<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, LegacyArray, { print_array_like(f, ArrayLike::LegacyArray(self)) })
    }
}

impl<'arena, A> Format<'arena, A> for List<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, List, { print_array_like(f, ArrayLike::List(self)) })
    }
}

impl<'arena, A> Format<'arena, A> for ArrayElement<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
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

impl<'arena, A> Format<'arena, A> for KeyValueArrayElement<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, KeyValueArrayElement, {
            let lhs = self.key.format(f);
            let operator = Document::String(b"=>");

            Document::Group(Group::new(vec_in![f.arena; print_assignment_with_alignment(
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

impl<'arena, A> Format<'arena, A> for ValueArrayElement<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ValueArrayElement, { self.value.format(f) })
    }
}

impl<'arena, A> Format<'arena, A> for VariadicArrayElement<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, VariadicArrayElement, {
            Document::Array(vec_in![f.arena; Document::String(b"..."), self.value.format(f)])
        })
    }
}

impl<'arena, A> Format<'arena, A> for MissingArrayElement
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, MissingArrayElement, { Document::empty() })
    }
}

impl<'arena, A> Format<'arena, A> for Construct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
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

impl<'arena, A> Format<'arena, A> for IssetConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IssetConstruct, {
            let mut contents = vec_in![f.arena; self.isset.format(f), Document::String(b"(")];

            if !self.values.is_empty() {
                let mut values = Document::join(f.arena, self.values.iter().map(|v| v.format(f)), Separator::CommaLine);

                if f.settings.trailing_comma {
                    values.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(b","))));
                }

                values.insert(0, Document::Line(Line::soft()));

                contents.push(Document::Indent(values));
                contents.push(Document::Line(Line::soft()));
            }

            contents.push(Document::String(b")"));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena, A> Format<'arena, A> for EmptyConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, EmptyConstruct, {
            Document::Group(Group::new(vec_in![f.arena;
                self.empty.format(f),
                Document::String(b"("),
                self.value.format(f),
                Document::String(b")"),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for EvalConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, EvalConstruct, {
            Document::Group(Group::new(vec_in![f.arena;
                self.eval.format(f),
                Document::String(b"("),
                self.value.format(f),
                Document::String(b")"),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for IncludeConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IncludeConstruct, {
            Document::Group(Group::new(vec_in![f.arena;
                self.include.format(f),
                Document::Indent(vec_in![f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for IncludeOnceConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, IncludeOnceConstruct, {
            Document::Group(Group::new(vec_in![f.arena;
                self.include_once.format(f),
                Document::Indent(vec_in![f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for RequireConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, RequireConstruct, {
            Document::Group(Group::new(vec_in![f.arena;
                self.require.format(f),
                Document::Indent(vec_in![f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for RequireOnceConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, RequireOnceConstruct, {
            Document::Group(Group::new(vec_in![f.arena;
                self.require_once.format(f),
                Document::Indent(vec_in![f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for PrintConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, PrintConstruct, {
            Document::Group(Group::new(vec_in![f.arena;
                self.print.format(f),
                Document::Indent(vec_in![f.arena; Document::Line(Line::default()), self.value.format(f)]),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for ExitConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ExitConstruct, { print_call_like_node(f, CallLikeNode::ExitConstruct(self)) })
    }
}

impl<'arena, A> Format<'arena, A> for DieConstruct<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, DieConstruct, { print_call_like_node(f, CallLikeNode::DieConstruct(self)) })
    }
}

impl<'arena, A> Format<'arena, A> for Argument<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Argument, {
            match self {
                Argument::Positional(a) => a.format(f),
                Argument::Named(a) => a.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for PositionalArgument<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, PositionalArgument, {
            match self.ellipsis {
                Some(_) => {
                    Document::Group(Group::new(vec_in![f.arena; Document::String(b"..."), self.value.format(f)]))
                }
                None => Document::Group(Group::new(vec_in![f.arena; self.value.format(f)])),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for NamedArgument<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, NamedArgument, {
            let padding = if let Some(padding) = f.argument_state.named_argument_padding {
                let mut spaces = BumpVec::with_capacity_in(padding, f.arena);
                spaces.resize(padding, b' ');
                Document::String(spaces.leak())
            } else {
                Document::empty()
            };

            Document::Group(Group::new(vec_in![f.arena;
                self.name.format(f),
                padding,
                Document::String(b":"),
                Document::space(),
                self.value.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for PartialArgumentList<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, PartialArgumentList, { print_partial_argument_list(f, self) })
    }
}

impl<'arena, A> Format<'arena, A> for PartialArgument<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
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

impl<'arena, A> Format<'arena, A> for PlaceholderArgument
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, PlaceholderArgument, { Document::String(b"?") })
    }
}

impl<'arena, A> Format<'arena, A> for VariadicPlaceholderArgument
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, VariadicPlaceholderArgument, { Document::String(b"...") })
    }
}

impl<'arena, A> Format<'arena, A> for NamedPlaceholderArgument<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, NamedPlaceholderArgument, {
            Document::Group(Group::new(vec_in![f.arena;
                self.name.format(f),
                Document::String(b":"),
                Document::space(),
                Document::String(b"?"),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for Assignment<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Assignment, {
            let lhs = self.lhs.format(f);

            let operator = match self.operator {
                AssignmentOperator::Assign(_) => Document::String(b"="),
                AssignmentOperator::Addition(_) => Document::String(b"+="),
                AssignmentOperator::Subtraction(_) => Document::String(b"-="),
                AssignmentOperator::Multiplication(_) => Document::String(b"*="),
                AssignmentOperator::Division(_) => Document::String(b"/="),
                AssignmentOperator::Modulo(_) => Document::String(b"%="),
                AssignmentOperator::Exponentiation(_) => Document::String(b"**="),
                AssignmentOperator::Concat(_) => Document::String(b".="),
                AssignmentOperator::BitwiseAnd(_) => Document::String(b"&="),
                AssignmentOperator::BitwiseOr(_) => Document::String(b"|="),
                AssignmentOperator::BitwiseXor(_) => Document::String(b"^="),
                AssignmentOperator::LeftShift(_) => Document::String(b"<<="),
                AssignmentOperator::RightShift(_) => Document::String(b">>="),
                AssignmentOperator::Coalesce(_) => Document::String(b"??="),
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

impl<'arena, A> Format<'arena, A> for ArrowFunction<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ArrowFunction, {
            let mut contents = vec_in![f.arena];
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
                contents.push(Document::String(b"&"));
            }

            contents.push(self.parameter_list.format(f));
            if let Some(h) = &self.return_type_hint {
                contents.push(h.format(f));
            }

            contents.push(Document::String(b" => "));
            contents.push(format_return_value(f, self.expression));

            Document::Group(Group::new(contents))
        })
    }
}

impl<'arena, A> Format<'arena, A> for ClassLikeMemberSelector<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ClassLikeMemberSelector, {
            match self {
                ClassLikeMemberSelector::Identifier(s) => s.format(f),
                ClassLikeMemberSelector::Variable(s) => s.format(f),
                ClassLikeMemberSelector::Expression(s) => s.format(f),
                ClassLikeMemberSelector::Missing(_) => Document::empty(),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for ClassLikeMemberExpressionSelector<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ClassLikeMemberExpressionSelector, {
            Document::Group(Group::new(
                vec_in![f.arena; Document::String(b"{"), self.expression.format(f), Document::String(b"}")],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for ClassLikeConstantSelector<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ClassLikeConstantSelector, {
            match self {
                ClassLikeConstantSelector::Identifier(s) => s.format(f),
                ClassLikeConstantSelector::Expression(s) => s.format(f),
                ClassLikeConstantSelector::Missing(_) => Document::empty(),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for ConstantAccess<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ConstantAccess, { self.name.format(f) })
    }
}

impl<'arena, A> Format<'arena, A> for Access<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
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

impl<'arena, A> Format<'arena, A> for PropertyAccess<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, PropertyAccess, {
            Document::Group(Group::new(
                vec_in![f.arena; self.object.format(f), Document::String(b"->"), self.property.format(f)],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for NullSafePropertyAccess<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, NullSafePropertyAccess, {
            Document::Group(Group::new(
                vec_in![f.arena; self.object.format(f), Document::String(b"?->"), self.property.format(f)],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for StaticPropertyAccess<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, StaticPropertyAccess, {
            Document::Group(Group::new(
                vec_in![f.arena; self.class.format(f), Document::String(b"::"), self.property.format(f)],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for ClassConstantAccess<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ClassConstantAccess, {
            Document::Group(Group::new(
                vec_in![f.arena; self.class.format(f), Document::String(b"::"), self.constant.format(f)],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for Call<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Call, { print_call_like_node(f, CallLikeNode::Call(self)) })
    }
}

impl<'arena, A> Format<'arena, A> for Throw<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Throw, {
            Document::Group(Group::new(
                vec_in![f.arena; self.throw.format(f), Document::space(), self.exception.format(f)],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for Instantiation<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Instantiation, { print_call_like_node(f, CallLikeNode::Instantiation(self)) })
    }
}

impl<'arena, A> Format<'arena, A> for ArrayAccess<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ArrayAccess, {
            Document::Group(Group::new(vec_in![f.arena;
                self.array.format(f),
                Document::String(b"["),
                self.index.format(f),
                Document::String(b"]"),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for ArrayAppend<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ArrayAppend, {
            Document::Group(Group::new(vec_in![f.arena; self.array.format(f), Document::String(b"[]")]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for MatchArm<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        format_match_arm(f, self, None)
    }
}

impl<'arena, A> Format<'arena, A> for MatchDefaultArm<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, MatchDefaultArm, {
            Document::Group(Group::new(vec_in![f.arena;
                self.default.format(f),
                format_token(f, self.arrow, b" => "),
                self.expression.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for MatchExpressionArm<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        format_match_expression_arm(f, self, None)
    }
}

#[derive(Debug, Clone, Copy)]
struct MatchArmAlignment {
    name_padding: usize,
}

fn format_match_arm<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    arm: &'arena MatchArm<'arena>,
    alignment: Option<MatchArmAlignment>,
) -> Document<'arena, A>
where
    A: Arena,
{
    wrap!(f, arm, MatchArm, {
        match arm {
            MatchArm::Expression(a) => format_match_expression_arm(f, a, alignment),
            MatchArm::Default(a) => format_match_default_arm(f, a, alignment),
        }
    })
}

fn format_match_default_arm<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    arm: &'arena MatchDefaultArm<'arena>,
    alignment: Option<MatchArmAlignment>,
) -> Document<'arena, A>
where
    A: Arena,
{
    wrap!(f, arm, MatchDefaultArm, {
        Document::Group(Group::new(vec_in![f.arena;
            arm.default.format(f),
            match_arm_alignment_padding(f, alignment),
            format_token(f, arm.arrow, b" => "),
            arm.expression.format(f),
        ]))
    })
}

fn format_match_expression_arm<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    arm: &'arena MatchExpressionArm<'arena>,
    alignment: Option<MatchArmAlignment>,
) -> Document<'arena, A>
where
    A: Arena,
{
    wrap!(f, arm, MatchExpressionArm, {
        let len = arm.conditions.len();

        let must_break = arm
            .conditions
            .iter()
            .take(len.saturating_sub(1))
            .any(|condition| f.has_comment(condition.span(), CommentFlags::TRAILING | CommentFlags::LINE));

        let mut contents = vec_in![f.arena];
        for (i, condition) in arm.conditions.iter().enumerate() {
            contents.push(condition.format(f));
            if i != (len - 1) {
                contents.push(Document::String(b","));
                contents.push(if must_break { Document::Line(Line::hard()) } else { Document::Line(Line::default()) });
            } else if f.settings.trailing_comma && i > 0 {
                contents.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(b","))));
            }
        }

        let group_id = f.next_id();
        contents.push(match_arm_alignment_padding_unless_breaks(f, alignment, group_id));
        contents.push(Document::IndentIfBreak(IndentIfBreak::new(
            group_id,
            vec_in![f.arena;
                if must_break { Document::Line(Line::hard()) } else { Document::Line(Line::default()) },
                format_token(f, arm.arrow, b"=> "),
            ],
        )));

        Document::Group(
            Group::new(vec_in![f.arena;
                Document::Group(Group::new(contents).with_break_mode(if must_break { BreakMode::Force } else { BreakMode::Auto })),
                arm.expression.format(f),
            ])
            .with_id(group_id)
            .with_break_mode(if must_break { BreakMode::Force } else { BreakMode::Auto }),
        )
    })
}

fn match_arm_alignment_padding<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    alignment: Option<MatchArmAlignment>,
) -> Document<'arena, A>
where
    A: Arena,
{
    match alignment {
        Some(MatchArmAlignment { name_padding }) if name_padding > 0 => {
            Document::String(f.as_str(" ".repeat(name_padding)))
        }
        _ => Document::empty(),
    }
}

fn match_arm_alignment_padding_unless_breaks<'arena, A>(
    f: &mut FormatterState<'_, 'arena, A>,
    alignment: Option<MatchArmAlignment>,
    group_id: GroupIdentifier,
) -> Document<'arena, A>
where
    A: Arena,
{
    if alignment.is_none_or(|alignment| alignment.name_padding == 0) {
        return Document::empty();
    }

    let padding = match_arm_alignment_padding(f, alignment);
    Document::IfBreak(IfBreak::new(f.arena, Document::empty(), padding).with_id(group_id))
}

fn detect_match_arm_alignment_runs<'arena, A>(
    f: &FormatterState<'_, 'arena, A>,
    arms: &'arena [MatchArm<'arena>],
) -> std::vec::Vec<AlignmentRun>
where
    A: Arena,
{
    if !f.settings.align_assignment_like || arms.is_empty() {
        return std::vec::Vec::new();
    }

    let mut runs = std::vec::Vec::new();
    let mut run_start: Option<usize> = None;

    for (i, arm) in arms.iter().enumerate() {
        let is_alignable = match_arm_lhs_width(arm).is_some();
        let should_break_run = run_start.is_some_and(|_start_idx| {
            if !is_alignable {
                return true;
            }

            if i > 0 {
                let prev_span = arms[i - 1].span();
                let curr_span = arm.span();
                if has_comment_between(f, prev_span, curr_span) {
                    return true;
                }
            }

            false
        });

        if should_break_run {
            if let Some(start_idx) = run_start
                && i - start_idx >= 2
            {
                let widths = calculate_match_arm_widths(&arms[start_idx..i]);
                runs.push(AlignmentRun::new(start_idx, i, widths));
            }
            run_start = None;
        }

        if is_alignable {
            if run_start.is_none() {
                run_start = Some(i);
            }
        } else {
            if let Some(start_idx) = run_start
                && i - start_idx >= 2
            {
                let widths = calculate_match_arm_widths(&arms[start_idx..i]);
                runs.push(AlignmentRun::new(start_idx, i, widths));
            }
            run_start = None;
        }
    }

    if let Some(start_idx) = run_start {
        let len = arms.len();
        if len - start_idx >= 2 {
            let widths = calculate_match_arm_widths(&arms[start_idx..]);
            runs.push(AlignmentRun::new(start_idx, len, widths));
        }
    }

    runs
}

fn calculate_match_arm_widths(arms: &[MatchArm<'_>]) -> AlignmentWidths {
    let max_name_width = arms.iter().filter_map(match_arm_lhs_width).max().unwrap_or(0);

    AlignmentWidths::new(max_name_width)
}

fn get_match_arm_alignment(runs: &[AlignmentRun], index: usize) -> Option<AlignmentWidths> {
    runs.iter().find(|run| run.contains(index)).map(|run| run.widths)
}

fn calculate_match_arm_alignment(arm: &MatchArm<'_>, widths: &AlignmentWidths) -> MatchArmAlignment {
    let current_width = match_arm_lhs_width(arm).unwrap_or(0);
    let name_padding = widths.name_width.saturating_sub(current_width);

    MatchArmAlignment { name_padding }
}

fn match_arm_lhs_width(arm: &MatchArm<'_>) -> Option<usize> {
    match arm {
        MatchArm::Expression(arm) => match_expression_arm_conditions_width(arm),
        MatchArm::Default(_) => Some("default".len()),
    }
}

fn match_expression_arm_conditions_width(arm: &MatchExpressionArm<'_>) -> Option<usize> {
    let mut width = 0usize;

    for (i, condition) in arm.conditions.iter().enumerate() {
        if i > 0 {
            width += 2;
        }

        width += get_expression_width(condition)?;
    }

    Some(width)
}

impl<'arena, A> Format<'arena, A> for Match<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Match, {
            let mut contents = vec_in![f.arena;
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

            contents.push(format_token(f, self.left_brace, b"{"));

            let should_break = self.arms.len() > 1
                || self.arms.iter().any(|arm| {
                    misc::has_new_line_in_range(
                        f.source_text,
                        arm.start_position().offset(),
                        arm.end_position().offset(),
                    )
                });

            if !self.arms.is_empty() {
                let alignment_runs = detect_match_arm_alignment_runs(f, self.arms.as_slice());
                let mut arms_document = Document::join(
                    f.arena,
                    self.arms.iter().enumerate().map(|(i, arm)| {
                        let alignment = get_match_arm_alignment(&alignment_runs, i)
                            .map(|widths| calculate_match_arm_alignment(arm, &widths));

                        format_match_arm(f, arm, alignment)
                    }),
                    if should_break { Separator::CommaHardLine } else { Separator::CommaLine },
                );

                if f.settings.trailing_comma {
                    if should_break {
                        arms_document.push(Document::String(b","));
                    } else {
                        arms_document.push(Document::IfBreak(IfBreak::then(f.arena, Document::String(b","))));
                    }
                }

                contents.push(Document::Indent(vec_in![f.arena;
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

            contents.push(format_token(f, self.right_brace, b"}"));

            Document::Group(Group::new(contents).with_break_mode(if should_break {
                BreakMode::Force
            } else {
                BreakMode::Auto
            }))
        })
    }
}

impl<'arena, A> Format<'arena, A> for Conditional<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Conditional, {
            let preserve_break = f.settings.preserve_breaking_conditional_expression && {
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
                    let condition_id = f.next_id();
                    let condition_doc = self.condition.format(f);
                    let question_doc = format_token_with_only_leading_comments(f, self.question_mark, b"? ");
                    let then_doc = then.format(f);
                    let colon_transition = if inline_colon {
                        if preserve_break {
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
                        Document::Line(Line::default())
                    };
                    let colon_doc = format_token_with_only_leading_comments(f, self.colon, b": ");
                    let else_doc = self.r#else.format(f);

                    let branches = Document::Indent(vec_in![f.arena;
                        Document::Line(Line::default()),
                        question_doc,
                        Document::Group(Group::new(vec_in![f.arena; then_doc]).with_id(then_id)),
                        colon_transition,
                        colon_doc,
                        else_doc,
                    ]);

                    let has_outer_indent_context = f.grandparent_node().is_some_and(|n| {
                        matches!(
                            n,
                            Node::Assignment(_)
                                | Node::PropertyItem(_)
                                | Node::ConstantItem(_)
                                | Node::Binary(_)
                                | Node::KeyValueArrayElement(_)
                                | Node::ValueArrayElement(_)
                                | Node::VariadicArrayElement(_)
                                | Node::PositionalArgument(_)
                                | Node::NamedArgument(_)
                                | Node::Return(_)
                                | Node::Throw(_)
                                | Node::Yield(_)
                        )
                    });
                    let tail = if has_outer_indent_context {
                        branches
                    } else {
                        Document::IndentIfBreak(IndentIfBreak::new(condition_id, vec_in![f.arena; branches]))
                    };

                    Document::Group(
                        Group::new(vec_in![f.arena;
                            Document::Group(Group::new(vec_in![f.arena; condition_doc]).with_id(condition_id)),
                            tail,
                        ])
                        .with_break_mode(if preserve_break { BreakMode::Preserve } else { BreakMode::Auto })
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

impl<'arena, A> Format<'arena, A> for CompositeString<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, CompositeString, {
            match self {
                CompositeString::ShellExecute(s) => s.format(f),
                CompositeString::Interpolated(s) => s.format(f),
                CompositeString::Document(s) => s.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for DocumentString<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, DocumentString, {
            let mut contents = vec_in![f.arena; Document::LineSuffixBoundary];
            if let Some(prefix) = &self.prefix {
                contents.push(Document::String(prefix.value));
            }
            contents.push(Document::String(b"<<<"));
            match self.kind {
                DocumentKind::Heredoc => {
                    contents.push(Document::String(self.label));
                }
                DocumentKind::Nowdoc => {
                    contents.push(Document::String(b"'"));
                    contents.push(Document::String(self.label));
                    contents.push(Document::String(b"'"));
                }
            }

            let indent = match self.indentation {
                DocumentIndentation::None => 0,
                DocumentIndentation::Whitespace(n) => n,
                DocumentIndentation::Tab(n) => n,
                DocumentIndentation::Mixed(t, w) => t + w,
            };

            let mut inner = vec_in![f.arena; Document::Line(Line::hard())];

            // Track the indentation from the last line of the previous literal part
            let mut last_part_indentation = Cow::Borrowed("");

            for part in &self.parts {
                let formatted = if let StringPart::Literal(l) = part {
                    let content = l.value;
                    let mut part_contents = vec_in![f.arena;];
                    let own_line = f.has_newline(l.span.start.offset, true);
                    let lines = f.split_lines(content);

                    for line in &lines {
                        let mut current = *line;
                        if own_line {
                            current = FormatterState::<'_, 'arena, A>::skip_leading_whitespace_up_to(current, indent);
                        }
                        let line = current;

                        let mut line_content = vec_in![f.arena; Document::String(line)];
                        if !line.is_empty() {
                            line_content.push(Document::DoNotTrim);
                        }

                        part_contents.push(Document::Array(line_content));
                    }

                    part_contents = Document::join(f.arena, part_contents, Separator::HardLine);

                    // if ends with a newline, add a newline
                    if content.ends_with(b"\n") {
                        part_contents.push(Document::Line(Line::hard()));
                    }

                    // Calculate indentation from the last line of this literal part
                    // We need to use the stripped line (after removing heredoc indent)
                    if let Some(last_line) = lines.last() {
                        let stripped_line = if own_line {
                            FormatterState::<'_, 'arena, A>::skip_leading_whitespace_up_to(last_line, indent)
                        } else {
                            *last_line
                        };

                        let mut tabs = 0;
                        let mut spaces = 0;
                        for &b in stripped_line.iter() {
                            match b {
                                b'\t' => tabs += 1,
                                b' ' => spaces += 1,
                                _ => break,
                            }
                        }

                        if tabs > 0 || spaces > 0 {
                            last_part_indentation = if tabs > 0 {
                                Cow::Owned(format!("{}{}", "\t".repeat(tabs), " ".repeat(spaces)))
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
                            DocumentIndentation::Mixed(t, w) => {
                                Cow::Owned(format!("{}{}", "\t".repeat(t), " ".repeat(w)))
                            }
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
                        contents: vec_in![f.arena;
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

impl<'arena, A> Format<'arena, A> for InterpolatedString<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, InterpolatedString, {
            let mut parts = vec_in![f.arena;];
            if let Some(prefix) = &self.prefix {
                parts.push(Document::String(prefix.value));
            }
            parts.push(Document::String(b"\""));
            let mut last_part_indentation = Cow::Borrowed("");

            for part in &self.parts {
                let formatted = match part {
                    StringPart::Literal(l) => {
                        let lines = f.split_lines(l.value);
                        if let Some(last_line) = lines.last() {
                            let mut tabs = 0;
                            let mut spaces = 0;
                            for &b in last_line.iter() {
                                match b {
                                    b'\t' => tabs += 1,
                                    b' ' => spaces += 1,
                                    _ => break,
                                }
                            }
                            if tabs > 0 || spaces > 0 {
                                last_part_indentation = if tabs > 0 {
                                    Cow::Owned(format!("{}{}", "\t".repeat(tabs), " ".repeat(spaces)))
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
                                contents: vec_in![f.arena; part.format(f)],
                            })
                        }
                    }
                };
                parts.push(formatted);
            }

            parts.push(Document::String(b"\""));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena, A> Format<'arena, A> for ShellExecuteString<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, ShellExecuteString, {
            let mut parts = vec_in![f.arena; Document::String(b"`")];
            let mut last_part_indentation = Cow::Borrowed("");

            for part in &self.parts {
                let formatted = match part {
                    StringPart::Literal(l) => {
                        let lines = f.split_lines(l.value);
                        if let Some(last_line) = lines.last() {
                            let mut tabs = 0;
                            let mut spaces = 0;
                            for &b in last_line.iter() {
                                match b {
                                    b'\t' => tabs += 1,
                                    b' ' => spaces += 1,
                                    _ => break,
                                }
                            }
                            if tabs > 0 || spaces > 0 {
                                last_part_indentation = if tabs > 0 {
                                    Cow::Owned(format!("{}{}", "\t".repeat(tabs), " ".repeat(spaces)))
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
                                contents: vec_in![f.arena; part.format(f)],
                            })
                        }
                    }
                    _ => part.format(f),
                };
                parts.push(formatted);
            }

            parts.push(Document::String(b"`"));

            Document::Group(Group::new(parts))
        })
    }
}

impl<'arena, A> Format<'arena, A> for StringPart<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, StringPart, {
            match self {
                StringPart::Literal(s) => s.format(f),
                StringPart::Expression(s) => s.format(f),
                StringPart::BracedExpression(s) => s.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for LiteralStringPart<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, LiteralStringPart, {
            utils::replace_end_of_line(f, Document::String(self.value), Separator::LiteralLine, false)
        })
    }
}

impl<'arena, A> Format<'arena, A> for BracedExpressionStringPart<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, BracedExpressionStringPart, {
            Document::Group(Group::new(
                vec_in![f.arena; Document::String(b"{"), self.expression.format(f), Document::String(b"}")],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for Yield<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Yield, {
            match self {
                Yield::Value(y) => y.format(f),
                Yield::Pair(y) => y.format(f),
                Yield::From(y) => y.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for YieldValue<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, YieldValue, {
            match &self.value {
                Some(v) => Document::Group(Group::new(
                    vec_in![f.arena; self.r#yield.format(f), Document::space(), v.format(f)],
                )),
                None => self.r#yield.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for YieldPair<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, YieldPair, {
            let group_id = f.next_id();

            let yield_keyword = self.r#yield.format(f);
            let key_document = self.key.format(f);

            Document::Group(
                Group::new(vec_in![f.arena;
                    yield_keyword,
                    Document::space(),
                    print_assignment(
                        f,
                        AssignmentLikeNode::YieldPair(self),
                        key_document,
                        Document::String(b"=>"),
                        self.value,
                    ),
                ])
                .with_id(group_id),
            )
        })
    }
}

impl<'arena, A> Format<'arena, A> for YieldFrom<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, YieldFrom, {
            Document::Group(Group::new(vec_in![f.arena;
                self.r#yield.format(f),
                Document::space(),
                self.from.format(f),
                Document::space(),
                self.iterator.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for Clone<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, Clone, {
            Document::Group(Group::new(
                vec_in![f.arena; self.clone.format(f), Document::space(), self.object.format(f)],
            ))
        })
    }
}

impl<'arena, A> Format<'arena, A> for MagicConstant<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
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

impl<'arena, A> Format<'arena, A> for PartialApplication<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, PartialApplication, {
            match &self {
                PartialApplication::Function(p) => p.format(f),
                PartialApplication::Method(p) => p.format(f),
                PartialApplication::StaticMethod(p) => p.format(f),
            }
        })
    }
}

impl<'arena, A> Format<'arena, A> for FunctionPartialApplication<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, FunctionPartialApplication, {
            Document::Group(Group::new(vec_in![f.arena; self.function.format(f), self.argument_list.format(f)]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for MethodPartialApplication<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, MethodPartialApplication, {
            Document::Group(Group::new(vec_in![f.arena;
                self.object.format(f),
                Document::String(b"->"),
                self.method.format(f),
                self.argument_list.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for StaticMethodPartialApplication<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, StaticMethodPartialApplication, {
            Document::Group(Group::new(vec_in![f.arena;
                self.class.format(f),
                Document::String(b"::"),
                self.method.format(f),
                self.argument_list.format(f),
            ]))
        })
    }
}

impl<'arena, A> Format<'arena, A> for AnonymousClass<'arena>
where
    A: Arena,
{
    fn format(&'arena self, f: &mut FormatterState<'_, 'arena, A>) -> Document<'arena, A> {
        wrap!(f, self, AnonymousClass, {
            let new_doc = self.new.format(f);
            let signature_start = self.modifiers.first().map(|m| m.span()).unwrap_or_else(|| self.class.span());
            let inline_doc = f.collect_inline_block_comments_between(self.new.span(), signature_start);

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

            let body = print_class_like_body(f, &self.left_brace, &self.members, &self.right_brace, Some(signature_id));

            if let Some(attributes) = misc::print_attribute_list_sequence(f, &self.attribute_lists) {
                Document::Group(Group::new(vec_in![f.arena;
                    new_doc,
                    Document::Indent(vec_in![f.arena;
                        Document::Line(Line::hard()),
                        attributes,
                        Document::Line(Line::hard()),
                        signature,
                        body,
                    ]),
                ]))
            } else {
                let mut parts = vec_in![f.arena; new_doc, Document::space()];
                if let Some(doc) = inline_doc {
                    parts.push(doc);
                }
                parts.push(signature);
                parts.push(body);
                Document::Group(Group::new(parts))
            }
        })
    }
}
