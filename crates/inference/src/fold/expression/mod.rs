use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_oracle::id::SymbolId;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known::TYPE_NEVER;
use mago_span::Span;

use crate::flow::Flow;
use crate::fold::InferenceFolder;
use crate::fold::expression::construct::RequireKind;

pub mod annotation;
pub mod array;
pub mod assignment;
pub mod binary;
pub mod clone;
pub mod composite_string;
pub mod conditional;
pub mod constant;
pub mod construct;
pub mod list;
pub mod literal;
pub mod magic_constant;
pub mod r#match;
pub mod throw;
pub mod unary;
pub mod variable;
pub mod r#yield;

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_expression(
        &mut self,
        expression: &'source Expression<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        match &expression.kind {
            ExpressionKind::Parenthesized(inner) => self.infer_parenthesized_expression(expression.span, inner),
            ExpressionKind::Literal(literal) => self.infer_literal(expression.span, literal),
            ExpressionKind::Assignment(assignment) => self.infer_assignment(expression.span, assignment),
            ExpressionKind::Binary(binary) => self.infer_binary(expression.span, binary),
            ExpressionKind::UnaryPrefix(unary) => self.infer_unary_prefix(expression.span, unary),
            ExpressionKind::UnaryPostfix(unary) => self.infer_unary_postfix(expression.span, unary),
            ExpressionKind::Array(elements) => self.infer_array(expression.span, elements),
            ExpressionKind::Match(match_expression) => self.infer_match(expression.span, match_expression),
            ExpressionKind::Variable(variable) => self.infer_variable(expression.span, variable),
            ExpressionKind::CompositeString(parts) => self.infer_composite_string(expression.span, parts),
            ExpressionKind::ShellExecute(parts) => self.infer_shell_execute(expression.span, parts),
            ExpressionKind::Annotation(annotation) => self.infer_annotation(expression.span, annotation),
            ExpressionKind::Conditional(conditional) => self.infer_conditional(expression.span, conditional),
            ExpressionKind::List(delimited) => self.infer_list(expression.span, delimited),
            ExpressionKind::ArrayAppend(array) => {
                let array = self.infer_expression(array);

                self.diverging(expression.span, ExpressionKind::ArrayAppend(self.arena.alloc(array)))
            }
            ExpressionKind::Item(_item_expression) => todo!(),
            ExpressionKind::Call(_call) => todo!(),
            ExpressionKind::PartialApplication(_partial_application) => todo!(),
            ExpressionKind::Access(_access) => todo!(),
            ExpressionKind::Clone(operand) => self.infer_clone(expression.span, operand),
            ExpressionKind::Empty(operand) => self.infer_empty(expression.span, operand),
            ExpressionKind::Eval(operand) => self.infer_eval(expression.span, operand),
            ExpressionKind::Include(operand) => self.infer_require_like(expression.span, operand, RequireKind::Include),
            ExpressionKind::IncludeOnce(operand) => {
                self.infer_require_like(expression.span, operand, RequireKind::IncludeOnce)
            }
            ExpressionKind::Require(operand) => self.infer_require_like(expression.span, operand, RequireKind::Require),
            ExpressionKind::RequireOnce(operand) => {
                self.infer_require_like(expression.span, operand, RequireKind::RequireOnce)
            }
            ExpressionKind::Print(operand) => self.infer_print(expression.span, operand),
            ExpressionKind::Isset(delimited) => self.infer_isset(expression.span, delimited),
            ExpressionKind::Exit(arguments) => self.infer_exit(expression.span, arguments.as_ref()),
            ExpressionKind::MagicConstant(magic_constant) => self.infer_magic_constant(expression.span, magic_constant),
            ExpressionKind::Constant(identifier) => self.infer_constant(expression.span, identifier),
            ExpressionKind::Instantiation(_instantiation) => todo!(),
            ExpressionKind::Yield(yield_expression) => self.infer_yield(expression.span, yield_expression),
            ExpressionKind::Throw(operand) => self.infer_throw(expression.span, operand),
            ExpressionKind::Parent => self.diverging(expression.span, ExpressionKind::Parent),
            ExpressionKind::Self_ => self.diverging(expression.span, ExpressionKind::Self_),
            ExpressionKind::Static => self.diverging(expression.span, ExpressionKind::Static),
            ExpressionKind::Identifier(identifier) => {
                self.diverging(expression.span, ExpressionKind::Identifier(identifier.copy_into(self.arena)))
            }
            ExpressionKind::Error(error_span) => self.diverging(expression.span, ExpressionKind::Error(*error_span)),
        }
    }

    pub fn infer_parenthesized_expression(
        &mut self,
        span: Span,
        expression: &'source Expression<'source, SymbolId, S, E>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        let inner_expression = self.infer_expression(expression);

        Expression {
            meta: inner_expression.meta,
            span,
            kind: ExpressionKind::Parenthesized(self.arena.alloc(inner_expression)),
        }
    }

    /// Wraps an already-rebuilt kind that is invalid in a value position (a bare
    /// name, `self`/`parent`/`static`, or a parse error) as a `never`-typed node.
    fn diverging(
        &self,
        span: Span,
        kind: ExpressionKind<'arena, SymbolId, Flow, Type<'arena>>,
    ) -> Expression<'arena, SymbolId, Flow, Type<'arena>> {
        Expression { meta: TYPE_NEVER, span, kind }
    }
}
