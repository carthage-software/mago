use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::walker::Walker;

use crate::internal::context::Context;

pub mod checker;
pub mod consts;
pub mod context;

#[derive(Clone, Debug)]
pub struct CheckingWalker;

impl<'ast, 'arena> Walker<'ast, 'arena, Context<'_, 'ast, 'arena>> for CheckingWalker {
    #[inline]
    fn walk_in_statement(&self, statement: &'ast Statement<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        context.ancestors.push(statement.span());
    }

    #[inline]
    fn walk_in_expression(&self, expression: &'ast Expression<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        context.ancestors.push(expression.span());
    }

    #[inline]
    fn walk_out_statement(&self, _statement: &'ast Statement<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        context.ancestors.pop();
    }

    #[inline]
    fn walk_out_expression(&self, _expression: &'ast Expression<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        context.ancestors.pop();
    }

    #[inline]
    fn walk_in_program(&self, program: &'ast Program<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::statement::check_top_level_statements(program, context);
    }

    #[inline]
    fn walk_in_declare(&self, declare: &Declare<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::statement::check_declare(declare, context);
    }

    #[inline]
    fn walk_in_namespace(&self, namespace: &Namespace<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::statement::check_namespace(namespace, context);
    }

    #[inline]
    fn walk_in_hint(&self, hint: &Hint<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        context.hint_depth += 1;
        checker::hint::check_hint(hint, context);
    }

    #[inline]
    fn walk_out_hint(&self, _hint: &Hint, context: &mut Context<'_, 'ast, 'arena>) {
        context.hint_depth -= 1;
    }

    #[inline]
    fn walk_in_try(&self, r#try: &Try<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::r#try::check_try(r#try, context);
    }

    #[inline]
    fn walk_in_class(&self, class: &'ast Class<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::class_like::check_class(class, context);
    }

    #[inline]
    fn walk_in_interface(&self, interface: &'ast Interface<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::class_like::check_interface(interface, context);
    }

    #[inline]
    fn walk_in_trait(&self, r#trait: &'ast Trait<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::class_like::check_trait(r#trait, context);
    }

    #[inline]
    fn walk_in_enum(&self, r#enum: &'ast Enum<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::class_like::check_enum(r#enum, context);
    }

    #[inline]
    fn walk_in_anonymous_class(
        &self,
        anonymous_class: &'ast AnonymousClass<'arena>,
        context: &mut Context<'_, 'ast, 'arena>,
    ) {
        checker::class_like::check_anonymous_class(anonymous_class, context);
    }

    #[inline]
    fn walk_in_function(&self, function: &'ast Function<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::function_like::check_function(function, context);
    }

    #[inline]
    fn walk_in_attribute_list(
        &self,
        attribute_list: &'ast AttributeList<'arena>,
        context: &mut Context<'_, 'ast, 'arena>,
    ) {
        checker::attribute::check_attribute_list(attribute_list, context);
    }

    #[inline]
    fn walk_in_goto(&self, goto: &'ast Goto<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::statement::check_goto(goto, context);
    }

    #[inline]
    fn walk_in_argument_list(
        &self,
        argument_list: &'ast ArgumentList<'arena>,
        context: &mut Context<'_, 'ast, 'arena>,
    ) {
        checker::argument::check_argument_list(argument_list, context);
    }

    #[inline]
    fn walk_in_closure(&self, closure: &'ast Closure<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::function_like::check_closure(closure, context);
    }

    #[inline]
    fn walk_in_arrow_function(
        &self,
        arrow_function: &'ast ArrowFunction<'arena>,
        context: &mut Context<'_, 'ast, 'arena>,
    ) {
        checker::function_like::check_arrow_function(arrow_function, context);
    }

    #[inline]
    fn walk_in_function_like_parameter_list(
        &self,
        function_like_parameter_list: &'ast FunctionLikeParameterList<'arena>,
        context: &mut Context<'_, 'ast, 'arena>,
    ) {
        checker::function_like::check_parameter_list(function_like_parameter_list, context);
    }

    #[inline]
    fn walk_in_match(&self, r#match: &'ast Match<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::control_flow::check_match(r#match, context);
    }

    #[inline]
    fn walk_in_switch(&self, switch: &'ast Switch<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::control_flow::check_switch(switch, context);
    }

    #[inline]
    fn walk_in_assignment(&self, assignment: &'ast Assignment<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::assignment::check_assignment(assignment, context);
    }

    #[inline]
    fn walk_in_function_like_return_type_hint(
        &self,
        function_like_return_type_hint: &'ast FunctionLikeReturnTypeHint<'arena>,
        context: &mut Context<'_, 'ast, 'arena>,
    ) {
        checker::function_like::check_return_type_hint(function_like_return_type_hint, context);
    }

    #[inline]
    fn walk_in_closure_creation(
        &self,
        closure_creation: &'ast ClosureCreation<'arena>,
        context: &mut Context<'_, 'ast, 'arena>,
    ) {
        checker::closure_creation::check_closure_creation(closure_creation, context);
    }

    #[inline]
    fn walk_in_list(&self, list: &'ast List<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::array::check_list(list, context);
    }

    fn walk_in_call(&self, call: &'ast Call<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::call::check_call(call, context);
    }

    #[inline]
    fn walk_in_access(&self, access: &'ast Access<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::access::check_access(access, context);
    }

    #[inline]
    fn walk_in_unary_prefix_operator(
        &self,
        unary_prefix_operator: &'ast UnaryPrefixOperator<'arena>,
        context: &mut Context<'_, 'ast, 'arena>,
    ) {
        checker::expression::check_unary_prefix_operator(unary_prefix_operator, context);
    }

    #[inline]
    fn walk_literal_expression(&self, literal_expression: &'ast Literal, context: &mut Context<'_, 'ast, 'arena>) {
        checker::literal::check_literal(literal_expression, context);
    }

    #[inline]
    fn walk_in_constant(&self, constant: &'ast Constant<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::constant::check_constant(constant, context);
    }

    fn walk_in_pipe(&self, pipe: &'ast Pipe<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
        checker::pipe::check_pipe(pipe, context);
    }
}
