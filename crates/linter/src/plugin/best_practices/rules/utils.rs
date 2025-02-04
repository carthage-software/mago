use mago_ast::*;
use mago_interner::StringIdentifier;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::plugin::best_practices::rules::utils::internal::FunctionCallWalker;

/// Determine if a variable is a super global variable.
pub fn is_super_global_variable(name: &str) -> bool {
    matches!(
        name,
        "$_GET" | "$_POST" | "$_COOKIE" | "$_REQUEST" | "$_SERVER" | "$_FILES" | "$_ENV" | "$_SESSION" | "$GLOBALS"
    )
}

/// Determine if a variable is a predefined variable.
///
/// Predefined variables are variables that are defined by PHP.
pub fn is_predefined_variable(name: &str) -> bool {
    is_super_global_variable(name) || "$this" == name
}

/// Determine if the block potentially contains a function call with the given name.
///
/// If this function returns true, it means there is a high possibility that the block contains
/// a call to the given function. If this function returns false, it means that we are certain
/// that the block does not contain a call to the given function.
///
/// The reason why this function is not certain is because it is difficult to determine if a
/// call refers to the exact function we are looking for. For example, the following code:
///
/// ```php
/// <?php
///
/// namespace Foo;
///
/// function bar() {
///    return baz();
/// }
/// ```
///
/// `baz` could refer to the `baz` function in the `Foo` namespace or the `baz` function in the
/// global namespace. This function will return true in this case because it is difficult to
/// determine which `baz` function is being called.
///
/// Note that if the function is called with an alias, this function will return true. For example:
///
/// ```php
/// <?php
///
/// namespace Foo;
///
/// use function baz as someOtherName;
///
/// function bar() {
///    return someOtherName();
/// }
/// ```
///
/// This function will return true when called with `baz` as the function name.
pub fn potentially_contains_function_call(block: &Block, function_name: &str, context: &LintContext<'_>) -> bool {
    let mut context = (false, context);

    FunctionCallWalker(function_name).walk_block(block, &mut context);

    context.0
}

/// A helper function that determines if an expression potentially contains a function call with the given name.
///
/// This function is similar to `potentially_contains_function_call` but it works on expressions instead of blocks.
pub fn expression_potentially_contains_function_call(
    expression: &Expression,
    function_name: &str,
    context: &LintContext<'_>,
) -> bool {
    let mut context = (false, context);

    FunctionCallWalker(function_name).walk_expression(expression, &mut context);

    context.0
}

/// Determine if the expression uses the given variable.
pub fn is_variable_used_in_expression(
    expression: &Expression,
    context: &LintContext<'_>,
    variable: StringIdentifier,
) -> bool {
    use crate::plugin::best_practices::rules::utils::internal::VariableReference;
    use crate::plugin::best_practices::rules::utils::internal::VariableWalker;

    let mut context = (Vec::default(), context);

    VariableWalker.walk_expression(expression, &mut context);

    let variables = context.0;
    let mut reassigned = false;
    for variable_reference in variables {
        match variable_reference {
            VariableReference::Use(string_identifier) => {
                if !reassigned && string_identifier == variable {
                    return true;
                }
            }
            VariableReference::Assign(string_identifier) => {
                // the variable was re-assigned before it was used
                if string_identifier == variable {
                    reassigned = true;
                }
            }
            VariableReference::Unset(string_identifier) => {
                // the variable was unset before it was used
                if string_identifier == variable {
                    if reassigned {
                        reassigned = false;
                    } else {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Given a block, get all the variable names that are used in the block before they are declared.
pub fn get_foreign_variable_names(block: &Block, context: &LintContext<'_>) -> Vec<StringIdentifier> {
    use crate::plugin::best_practices::rules::utils::internal::VariableReference;
    use crate::plugin::best_practices::rules::utils::internal::VariableWalker;

    // we must use a vec here instead of a set because we need to preserve the order of the variables
    // in order to determine the order in which they are used/delcared.
    let mut context = (Vec::default(), context);

    VariableWalker.walk_block(block, &mut context);

    let variables = context.0;
    let mut assigned = Vec::new();
    let mut foreign = Vec::new();
    for variable in variables {
        match &variable {
            VariableReference::Use(string_identifier) => {
                if !assigned.contains(string_identifier) && !foreign.contains(string_identifier) {
                    // the variable is used before it is assigned
                    foreign.push(*string_identifier);
                }
            }
            VariableReference::Assign(string_identifier) => {
                assigned.push(*string_identifier);
            }
            VariableReference::Unset(string_identifier) => {
                assigned.retain(|assigned| assigned != string_identifier);
            }
        }
    }

    foreign
}

mod internal {
    use super::is_predefined_variable;

    use mago_ast::*;
    use mago_interner::StringIdentifier;
    use mago_walker::Walker;

    use crate::context::LintContext;

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
    pub(super) enum VariableReference {
        Use(StringIdentifier),
        Assign(StringIdentifier),
        Unset(StringIdentifier),
    }

    #[derive(Debug)]
    pub(super) struct VariableWalker;

    #[derive(Debug)]
    pub(super) struct FunctionCallWalker<'a>(pub &'a str);

    impl<'a> Walker<(Vec<VariableReference>, &'a LintContext<'a>)> for VariableWalker {
        fn walk_in_foreach_value_target<'ast>(
            &self,
            foreach_value_target: &'ast ForeachValueTarget,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            scan_expression_for_assignment(&foreach_value_target.value, context.1, &mut context.0);
        }

        fn walk_in_foreach_key_value_target<'ast>(
            &self,
            foreach_key_value_target: &'ast ForeachKeyValueTarget,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            scan_expression_for_assignment(&foreach_key_value_target.key, context.1, &mut context.0);
            scan_expression_for_assignment(&foreach_key_value_target.value, context.1, &mut context.0);
        }

        fn walk_in_try_catch_clause<'ast>(
            &self,
            try_catch_clause: &'ast TryCatchClause,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            if let Some(variable) = &try_catch_clause.variable {
                context.0.push(VariableReference::Assign(variable.name));
            }
        }

        fn walk_in_static_concrete_item<'ast>(
            &self,
            static_concrete_item: &'ast StaticConcreteItem,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            context.0.push(VariableReference::Assign(static_concrete_item.variable.name));
        }

        fn walk_in_static_abstract_item<'ast>(
            &self,
            static_abstract_item: &'ast StaticAbstractItem,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            context.0.push(VariableReference::Assign(static_abstract_item.variable.name));
        }

        fn walk_in_global<'ast>(
            &self,
            global: &'ast Global,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            for variable in global.variables.iter() {
                let Variable::Direct(variable) = variable else {
                    continue;
                };

                context.0.push(VariableReference::Assign(variable.name));
            }
        }

        fn walk_assignment<'ast>(
            &self,
            assignment: &'ast Assignment,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            // we need to walk the right hand side first to ensure that we don't
            // mark variables as being assigned to when they are only being used
            // in the right hand side.
            self.walk_expression(&assignment.rhs, context);

            let mut variables = Vec::default();
            scan_expression_for_assignment(&assignment.lhs, context.1, &mut variables);

            match assignment.operator {
                AssignmentOperator::Assign(_) => {
                    context.0.extend(variables);
                }
                _ => {
                    for variable in variables {
                        if let VariableReference::Assign(name) = variable {
                            context.0.push(VariableReference::Use(name));
                            context.0.push(VariableReference::Assign(name));
                        } else {
                            context.0.push(variable);
                        }
                    }
                }
            }

            // then we walk the left hand side
            self.walk_expression(&assignment.lhs, context);
        }

        fn walk_in_direct_variable<'ast>(
            &self,
            direct_variable: &'ast DirectVariable,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            let name = context.1.interner.lookup(&direct_variable.name);
            if !is_predefined_variable(name) {
                context.0.push(VariableReference::Use(direct_variable.name));
            }
        }

        fn walk_closure<'ast>(
            &self,
            closure: &'ast Closure,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            if let Some(use_clause) = &closure.use_clause {
                for use_clause_variable in use_clause.variables.iter() {
                    context.0.push(VariableReference::Use(use_clause_variable.variable.name));
                }
            }
        }

        fn walk_in_arrow_function<'ast>(
            &self,
            arrow_function: &'ast ArrowFunction,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            for parameter in arrow_function.parameter_list.parameters.iter() {
                context.0.push(VariableReference::Assign(parameter.variable.name));
            }
        }

        fn walk_out_arrow_function<'ast>(
            &self,
            arrow_function: &'ast ArrowFunction,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            for parameter in arrow_function.parameter_list.parameters.iter() {
                context.0.push(VariableReference::Unset(parameter.variable.name));
            }
        }

        #[inline(always)]
        fn walk_anonymous_class<'ast>(
            &self,
            anonymous_class: &'ast AnonymousClass,
            context: &mut (Vec<VariableReference>, &'a LintContext<'a>),
        ) {
            if let Some(argument_list) = anonymous_class.arguments.as_ref() {
                self.walk_argument_list(argument_list, context);
            }
        }

        #[inline(always)]
        fn walk_namespace<'ast>(&self, _: &'ast Namespace, _: &mut (Vec<VariableReference>, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_class<'ast>(&self, _: &'ast Class, _: &mut (Vec<VariableReference>, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_interface<'ast>(&self, _: &'ast Interface, _: &mut (Vec<VariableReference>, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_trait<'ast>(&self, _: &'ast Trait, _: &mut (Vec<VariableReference>, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_enum<'ast>(&self, _: &'ast Enum, _: &mut (Vec<VariableReference>, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_function<'ast>(&self, _: &'ast Function, _: &mut (Vec<VariableReference>, &'a LintContext<'a>)) {}
    }

    impl<'a> Walker<(bool, &'a LintContext<'a>)> for FunctionCallWalker<'a> {
        fn walk_in_function_call<'ast>(
            &self,
            function_call: &'ast FunctionCall,
            context: &mut (bool, &'a LintContext<'a>),
        ) {
            // we determined that the function is called already
            // so we can skip the rest of the function call
            if context.0 {
                return;
            }

            let Expression::Identifier(function_identifier) = function_call.function.as_ref() else {
                return;
            };

            let function_name = context.1.resolve_function_name(function_identifier);

            context.0 = self.0.eq(function_name);
        }

        #[inline(always)]
        fn walk_closure<'ast>(&self, _: &'ast Closure, _: &mut (bool, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_arrow_function<'ast>(&self, _: &'ast ArrowFunction, _: &mut (bool, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_namespace<'ast>(&self, _: &'ast Namespace, _: &mut (bool, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_class<'ast>(&self, _: &'ast Class, _: &mut (bool, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_interface<'ast>(&self, _: &'ast Interface, _: &mut (bool, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_trait<'ast>(&self, _: &'ast Trait, _: &mut (bool, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_enum<'ast>(&self, _: &'ast Enum, _: &mut (bool, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_function<'ast>(&self, _: &'ast Function, _: &mut (bool, &'a LintContext<'a>)) {}

        #[inline(always)]
        fn walk_anonymous_class<'ast>(&self, _: &'ast AnonymousClass, _: &mut (bool, &'a LintContext<'a>)) {}
    }

    fn scan_expression_for_assignment(
        expression: &Expression,
        context: &LintContext<'_>,
        variables: &mut Vec<VariableReference>,
    ) {
        match &expression {
            Expression::Variable(variable) => {
                let Variable::Direct(variable) = variable else {
                    return;
                };

                let name = context.interner.lookup(&variable.name);
                if !is_predefined_variable(name) {
                    variables.push(VariableReference::Assign(variable.name));
                }
            }
            Expression::Array(array) => {
                for element in array.elements.iter() {
                    match &element {
                        ArrayElement::KeyValue(key_value_array_element) => {
                            scan_expression_for_assignment(&key_value_array_element.key, context, variables);
                            scan_expression_for_assignment(&key_value_array_element.value, context, variables);
                        }
                        ArrayElement::Value(value_array_element) => {
                            scan_expression_for_assignment(&value_array_element.value, context, variables);
                        }
                        _ => {}
                    }
                }
            }
            Expression::LegacyArray(array) => {
                for element in array.elements.iter() {
                    match &element {
                        ArrayElement::KeyValue(key_value_array_element) => {
                            scan_expression_for_assignment(&key_value_array_element.key, context, variables);
                            scan_expression_for_assignment(&key_value_array_element.value, context, variables);
                        }
                        ArrayElement::Value(value_array_element) => {
                            scan_expression_for_assignment(&value_array_element.value, context, variables);
                        }
                        _ => {}
                    }
                }
            }
            Expression::List(list) => {
                for element in list.elements.iter() {
                    match &element {
                        ArrayElement::KeyValue(key_value_array_element) => {
                            scan_expression_for_assignment(&key_value_array_element.key, context, variables);
                            scan_expression_for_assignment(&key_value_array_element.value, context, variables);
                        }
                        ArrayElement::Value(value_array_element) => {
                            scan_expression_for_assignment(&value_array_element.value, context, variables);
                        }
                        _ => {}
                    }
                }
            }
            Expression::ArrayAppend(append) => {
                if let Expression::Variable(Variable::Direct(variable)) = append.array.as_ref() {
                    let name = context.interner.lookup(&variable.name);
                    if !is_predefined_variable(name) {
                        variables.push(VariableReference::Use(variable.name));
                    }
                }

                scan_expression_for_assignment(&append.array, context, variables);
            }
            Expression::ArrayAccess(access) => {
                if let Expression::Variable(Variable::Direct(variable)) = access.array.as_ref() {
                    let name = context.interner.lookup(&variable.name);
                    if !is_predefined_variable(name) {
                        variables.push(VariableReference::Use(variable.name));
                    }
                }

                scan_expression_for_assignment(&access.array, context, variables);
                scan_expression_for_assignment(&access.index, context, variables);
            }
            _ => {}
        }
    }
}
