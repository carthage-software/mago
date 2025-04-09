use mago_reflection::CodebaseReflection;
use mago_reflection::class_like::ClassLikeReflection;
use mago_span::HasSpan;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::internal::checker;
use crate::internal::context::Context;
use crate::internal::reflector::class_like::reflect_anonymous_class;
use crate::internal::reflector::class_like::reflect_class;
use crate::internal::reflector::class_like::reflect_enum;
use crate::internal::reflector::class_like::reflect_interface;
use crate::internal::reflector::class_like::reflect_trait;
use crate::internal::reflector::constant::reflect_constant;
use crate::internal::reflector::constant::reflect_defined_constant;
use crate::internal::reflector::function_like::reflect_arrow_function;
use crate::internal::reflector::function_like::reflect_closure;
use crate::internal::reflector::function_like::reflect_function;

#[derive(Clone, Debug)]
pub struct ModuleBuildingWalker {
    pub reflection: CodebaseReflection,
    scope: Vec<ClassLikeReflection>,
}

impl ModuleBuildingWalker {
    pub fn new() -> Self {
        Self { reflection: CodebaseReflection::new(), scope: vec![] }
    }
}

impl MutWalker<Context<'_>> for ModuleBuildingWalker {
    #[inline]
    fn walk_in_statement(&mut self, statement: &Statement, context: &mut Context<'_>) {
        context.ancestors.push(statement.span());
    }

    #[inline]
    fn walk_in_expression(&mut self, expression: &Expression, context: &mut Context<'_>) {
        context.ancestors.push(expression.span());
    }

    #[inline]
    fn walk_out_statement(&mut self, _statement: &Statement, context: &mut Context<'_>) {
        context.ancestors.pop();
    }

    #[inline]
    fn walk_out_expression(&mut self, _expression: &Expression, context: &mut Context<'_>) {
        context.ancestors.pop();
    }

    #[inline]
    fn walk_in_program(&mut self, program: &Program, context: &mut Context<'_>) {
        checker::statement::check_top_level_statements(program, context);
    }

    #[inline]
    fn walk_opening_tag(&mut self, opening_tag: &OpeningTag, context: &mut Context<'_>) {
        checker::statement::check_opening_tag(opening_tag, context);
    }

    #[inline]
    fn walk_in_declare(&mut self, declare: &Declare, context: &mut Context<'_>) {
        checker::statement::check_declare(declare, context);
    }

    #[inline]
    fn walk_in_namespace(&mut self, namespace: &Namespace, context: &mut Context<'_>) {
        checker::statement::check_namespace(namespace, context);
    }

    #[inline]
    fn walk_in_hint(&mut self, hint: &Hint, context: &mut Context<'_>) {
        context.hint_depth += 1;
        checker::hint::check_hint(hint, context);
    }

    #[inline]
    fn walk_out_hint(&mut self, _hint: &Hint, context: &mut Context<'_>) {
        context.hint_depth -= 1;
    }

    #[inline]
    fn walk_in_try(&mut self, r#try: &Try, context: &mut Context<'_>) {
        checker::r#try::check_try(r#try, context);
    }

    #[inline]
    fn walk_in_class(&mut self, class: &Class, context: &mut Context<'_>) {
        self.scope.push(reflect_class(class, context));
        checker::class_like::check_class(class, context);
    }

    #[inline]
    fn walk_out_class(&mut self, _class: &Class, context: &mut Context<'_>) {
        if let Some(reflection) = self.scope.pop() {
            self.reflection.register_class_like(context.interner, reflection);
        }
    }

    #[inline]
    fn walk_in_interface(&mut self, interface: &Interface, context: &mut Context<'_>) {
        self.scope.push(reflect_interface(interface, context));
        checker::class_like::check_interface(interface, context);
    }

    #[inline]
    fn walk_out_interface(&mut self, _interface: &Interface, context: &mut Context<'_>) {
        if let Some(reflection) = self.scope.pop() {
            self.reflection.register_class_like(context.interner, reflection);
        }
    }

    #[inline]
    fn walk_in_trait(&mut self, r#trait: &Trait, context: &mut Context<'_>) {
        self.scope.push(reflect_trait(r#trait, context));
        checker::class_like::check_trait(r#trait, context);
    }

    #[inline]
    fn walk_out_trait(&mut self, _trait: &Trait, context: &mut Context<'_>) {
        if let Some(reflection) = self.scope.pop() {
            self.reflection.register_class_like(context.interner, reflection);
        }
    }

    #[inline]
    fn walk_in_enum(&mut self, r#enum: &Enum, context: &mut Context<'_>) {
        self.scope.push(reflect_enum(r#enum, context));
        checker::class_like::check_enum(r#enum, context);
    }

    #[inline]
    fn walk_out_enum(&mut self, _enum: &Enum, context: &mut Context<'_>) {
        if let Some(reflection) = self.scope.pop() {
            self.reflection.register_class_like(context.interner, reflection);
        }
    }

    #[inline]
    fn walk_in_anonymous_class(&mut self, anonymous_class: &AnonymousClass, context: &mut Context<'_>) {
        self.scope.push(reflect_anonymous_class(anonymous_class, context));
        checker::class_like::check_anonymous_class(anonymous_class, context);
    }

    #[inline]
    fn walk_out_anonymous_class(&mut self, _anonymous_class: &AnonymousClass, context: &mut Context<'_>) {
        if let Some(reflection) = self.scope.pop() {
            self.reflection.register_class_like(context.interner, reflection);
        };
    }

    #[inline]
    fn walk_in_function(&mut self, function: &Function, context: &mut Context<'_>) {
        if let Some(reflection) = reflect_function(function, context, self.scope.last()) {
            self.reflection.register_function_like(context.interner, reflection);
            checker::function_like::check_function(function, context);
        }
    }

    #[inline]
    fn walk_in_attribute_list(&mut self, attribute_list: &AttributeList, context: &mut Context<'_>) {
        checker::attribute::check_attribute_list(attribute_list, context);
    }

    #[inline]
    fn walk_in_goto(&mut self, goto: &Goto, context: &mut Context<'_>) {
        checker::statement::check_goto(goto, context);
    }

    #[inline]
    fn walk_in_argument_list(&mut self, argument_list: &ArgumentList, context: &mut Context<'_>) {
        checker::argument::check_argument_list(argument_list, context);
    }

    #[inline]
    fn walk_in_closure(&mut self, closure: &Closure, context: &mut Context<'_>) {
        self.reflection.register_function_like(context.interner, reflect_closure(closure, context, self.scope.last()));
        checker::function_like::check_closure(closure, context);
    }

    #[inline]
    fn walk_in_arrow_function(&mut self, arrow_function: &ArrowFunction, context: &mut Context<'_>) {
        self.reflection.register_function_like(
            context.interner,
            reflect_arrow_function(arrow_function, context, self.scope.last()),
        );
        checker::function_like::check_arrow_function(arrow_function, context);
    }

    #[inline]
    fn walk_in_function_like_parameter_list(
        &mut self,
        function_like_parameter_list: &FunctionLikeParameterList,
        context: &mut Context<'_>,
    ) {
        checker::function_like::check_parameter_list(function_like_parameter_list, context);
    }

    #[inline]
    fn walk_in_match(&mut self, r#match: &Match, context: &mut Context<'_>) {
        checker::control_flow::check_match(r#match, context);
    }

    #[inline]
    fn walk_in_switch(&mut self, switch: &Switch, context: &mut Context<'_>) {
        checker::control_flow::check_switch(switch, context);
    }

    #[inline]
    fn walk_in_assignment(&mut self, assignment: &Assignment, context: &mut Context<'_>) {
        checker::assignment::check_assignment(assignment, context);
    }

    #[inline]
    fn walk_in_function_like_return_type_hint(
        &mut self,
        function_like_return_type_hint: &FunctionLikeReturnTypeHint,
        context: &mut Context<'_>,
    ) {
        checker::function_like::check_return_type_hint(function_like_return_type_hint, context);
    }

    #[inline]
    fn walk_in_closure_creation(&mut self, closure_creation: &ClosureCreation, context: &mut Context<'_>) {
        checker::closure_creation::check_closure_creation(closure_creation, context);
    }

    #[inline]
    fn walk_in_list(&mut self, list: &List, context: &mut Context<'_>) {
        checker::array::check_list(list, context);
    }

    fn walk_in_call(&mut self, call: &Call, context: &mut Context<'_>) {
        checker::call::check_call(call, context);
    }

    #[inline]
    fn walk_in_function_call(&mut self, function_call: &FunctionCall, context: &mut Context<'_>) {
        if let Some(constant_reflection) = reflect_defined_constant(function_call, context) {
            self.reflection.register_constant(context.interner, constant_reflection);
        }
    }

    #[inline]
    fn walk_in_access(&mut self, access: &Access, context: &mut Context<'_>) {
        checker::access::check_access(access, context);
    }

    #[inline]
    fn walk_in_unary_prefix_operator(
        &mut self,
        unary_prefix_operator: &UnaryPrefixOperator,
        context: &mut Context<'_>,
    ) {
        checker::expression::check_unary_prefix_operator(unary_prefix_operator, context);
    }

    #[inline]
    fn walk_literal_expression(&mut self, literal_expression: &Literal, context: &mut Context<'_>) {
        checker::literal::check_literal(literal_expression, context);
    }

    #[inline]
    fn walk_in_constant(&mut self, constant: &Constant, context: &mut Context<'_>) {
        for reflection in reflect_constant(constant, context) {
            self.reflection.register_constant(context.interner, reflection);
        }

        checker::constant::check_constant(constant, context);
    }
}
