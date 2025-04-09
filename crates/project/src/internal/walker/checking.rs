#![allow(clippy::too_many_arguments)]

use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::walker::Walker;

use crate::internal::checker;
use crate::internal::context::Context;

#[derive(Clone, Debug)]
pub struct ModuleCheckingWalker;

impl Walker<Context<'_>> for ModuleCheckingWalker {
    #[inline]
    fn walk_in_statement(&self, statement: &Statement, context: &mut Context<'_>) {
        context.ancestors.push(statement.span());
    }

    #[inline]
    fn walk_in_expression(&self, expression: &Expression, context: &mut Context<'_>) {
        context.ancestors.push(expression.span());
    }

    #[inline]
    fn walk_out_statement(&self, _statement: &Statement, context: &mut Context<'_>) {
        context.ancestors.pop();
    }

    #[inline]
    fn walk_out_expression(&self, _expression: &Expression, context: &mut Context<'_>) {
        context.ancestors.pop();
    }

    #[inline]
    fn walk_in_program(&self, program: &Program, context: &mut Context<'_>) {
        checker::statement::check_top_level_statements(program, context);
    }

    #[inline]
    fn walk_opening_tag(&self, opening_tag: &OpeningTag, context: &mut Context<'_>) {
        checker::statement::check_opening_tag(opening_tag, context);
    }

    #[inline]
    fn walk_in_declare(&self, declare: &Declare, context: &mut Context<'_>) {
        checker::statement::check_declare(declare, context);
    }

    #[inline]
    fn walk_in_namespace(&self, namespace: &Namespace, context: &mut Context<'_>) {
        checker::statement::check_namespace(namespace, context);
    }

    #[inline]
    fn walk_in_hint(&self, hint: &Hint, context: &mut Context<'_>) {
        context.hint_depth += 1;
        checker::hint::check_hint(hint, context);
    }

    #[inline]
    fn walk_out_hint(&self, _hint: &Hint, context: &mut Context<'_>) {
        context.hint_depth -= 1;
    }

    #[inline]
    fn walk_in_try(&self, r#try: &Try, context: &mut Context<'_>) {
        checker::r#try::check_try(r#try, context);
    }

    #[inline]
    fn walk_in_class(&self, class: &Class, context: &mut Context<'_>) {
        checker::class_like::check_class(class, context);
    }

    #[inline]
    fn walk_in_interface(&self, interface: &Interface, context: &mut Context<'_>) {
        checker::class_like::check_interface(interface, context);
    }

    #[inline]
    fn walk_in_trait(&self, r#trait: &Trait, context: &mut Context<'_>) {
        checker::class_like::check_trait(r#trait, context);
    }

    #[inline]
    fn walk_in_enum(&self, r#enum: &Enum, context: &mut Context<'_>) {
        checker::class_like::check_enum(r#enum, context);
    }

    #[inline]
    fn walk_in_anonymous_class(&self, anonymous_class: &AnonymousClass, context: &mut Context<'_>) {
        checker::class_like::check_anonymous_class(anonymous_class, context);
    }

    #[inline]
    fn walk_in_function(&self, function: &Function, context: &mut Context<'_>) {
        checker::function_like::check_function(function, context);
    }

    #[inline]
    fn walk_in_attribute_list(&self, attribute_list: &AttributeList, context: &mut Context<'_>) {
        checker::attribute::check_attribute_list(attribute_list, context);
    }

    #[inline]
    fn walk_in_goto(&self, goto: &Goto, context: &mut Context<'_>) {
        checker::statement::check_goto(goto, context);
    }

    #[inline]
    fn walk_in_argument_list(&self, argument_list: &ArgumentList, context: &mut Context<'_>) {
        checker::argument::check_argument_list(argument_list, context);
    }

    #[inline]
    fn walk_in_closure(&self, closure: &Closure, context: &mut Context<'_>) {
        checker::function_like::check_closure(closure, context);
    }

    #[inline]
    fn walk_in_arrow_function(&self, arrow_function: &ArrowFunction, context: &mut Context<'_>) {
        checker::function_like::check_arrow_function(arrow_function, context);
    }

    #[inline]
    fn walk_in_function_like_parameter_list(
        &self,
        function_like_parameter_list: &FunctionLikeParameterList,
        context: &mut Context<'_>,
    ) {
        checker::function_like::check_parameter_list(function_like_parameter_list, context);
    }

    #[inline]
    fn walk_in_match(&self, r#match: &Match, context: &mut Context<'_>) {
        checker::control_flow::check_match(r#match, context);
    }

    #[inline]
    fn walk_in_switch(&self, switch: &Switch, context: &mut Context<'_>) {
        checker::control_flow::check_switch(switch, context);
    }

    #[inline]
    fn walk_in_assignment(&self, assignment: &Assignment, context: &mut Context<'_>) {
        checker::assignment::check_assignment(assignment, context);
    }

    #[inline]
    fn walk_in_function_like_return_type_hint(
        &self,
        function_like_return_type_hint: &FunctionLikeReturnTypeHint,
        context: &mut Context<'_>,
    ) {
        checker::function_like::check_return_type_hint(function_like_return_type_hint, context);
    }

    #[inline]
    fn walk_in_closure_creation(&self, closure_creation: &ClosureCreation, context: &mut Context<'_>) {
        checker::closure_creation::check_closure_creation(closure_creation, context);
    }

    #[inline]
    fn walk_in_list(&self, list: &List, context: &mut Context<'_>) {
        checker::array::check_list(list, context);
    }

    fn walk_in_call(&self, call: &Call, context: &mut Context<'_>) {
        checker::call::check_call(call, context);
    }

    #[inline]
    fn walk_in_access(&self, access: &Access, context: &mut Context<'_>) {
        checker::access::check_access(access, context);
    }

    #[inline]
    fn walk_in_unary_prefix_operator(&self, unary_prefix_operator: &UnaryPrefixOperator, context: &mut Context<'_>) {
        checker::expression::check_unary_prefix_operator(unary_prefix_operator, context);
    }

    #[inline]
    fn walk_literal_expression(&self, literal_expression: &Literal, context: &mut Context<'_>) {
        checker::literal::check_literal(literal_expression, context);
    }

    #[inline]
    fn walk_in_constant(&self, constant: &Constant, context: &mut Context<'_>) {
        checker::constant::check_constant(constant, context);
    }
}
