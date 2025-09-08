use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::FunctionLikeParameter;

use crate::code::IssueCode;
use crate::context::Context;
use crate::heuristic::unused_parameter::utils::expression_potentially_contains_function_call;
use crate::heuristic::unused_parameter::utils::get_foreign_variable_names;
use crate::heuristic::unused_parameter::utils::is_variable_used_in_expression;
use crate::statement::function_like::FunctionLikeBody;

const FUNC_GET_ARGS: &str = "func_get_args";

pub fn check_unused_params<'ctx, 'ast, 'arena>(
    metadata: &'ctx FunctionLikeMetadata,
    params: &'ast [FunctionLikeParameter<'arena>],
    body: FunctionLikeBody<'ast, 'arena>,
    ctx: &mut Context<'ctx, 'arena>,
) {
    match body {
        FunctionLikeBody::Statements(statements, _) => {
            if utils::potentially_contains_function_call(statements, FUNC_GET_ARGS, ctx) {
                // `func_get_args` is used, so we can't determine if the parameters are unused in this case
                return;
            }

            let foreign_variables = get_foreign_variable_names(statements, ctx);

            for param in params {
                if param.is_promoted_property() {
                    // Skip promoted properties
                    continue;
                }

                if foreign_variables.iter().any(|v| v.name == param.variable.name) {
                    continue;
                }

                report_parameter(
                    param,
                    metadata.span,
                    ctx,
                    if metadata.kind.is_closure() {
                        "closure"
                    } else if metadata.kind.is_method() {
                        "method"
                    } else {
                        "function"
                    },
                );
            }
        }
        FunctionLikeBody::Expression(expression) => {
            if expression_potentially_contains_function_call(expression, FUNC_GET_ARGS, ctx) {
                // `func_get_args` is used, so we can't determine if the parameters are unused in this case
                return;
            }

            for param in params {
                if !is_variable_used_in_expression(expression, ctx, param.variable.name) {
                    report_parameter(param, metadata.span, ctx, "arrow function");
                }
            }
        }
    }
}

fn report_parameter<'ctx, 'ast, 'arena>(
    parameter: &'ast FunctionLikeParameter<'arena>,
    function_like: Span,
    context: &mut Context<'ctx, 'arena>,
    kind: &'static str,
) {
    if parameter.ampersand.is_some() {
        return;
    }

    let parameter_name = parameter.variable.name;
    if parameter_name.starts_with("$_") {
        return;
    }

    let issue = Issue::help(format!("Parameter `{parameter_name}` is never used."))
        .with_code(IssueCode::UnusedParameter)
        .with_annotations([
            Annotation::primary(parameter.span()).with_message(format!("Parameter `{parameter_name}` is declared here.")),
            Annotation::secondary(function_like),
        ])
        .with_note(format!("This parameter is declared but not used within the {kind}."))
        .with_help("Consider prefixing the parameter with an underscore (`_`) to indicate that it is intentionally unused, or remove it if it is not needed.");

    context.collector.propose(issue, |plan| {
        plan.insert(
            parameter.variable.span().start.offset + 1, // skip the leading `$`
            "_",
            SafetyClassification::PotentiallyUnsafe,
        );
    });
}

mod utils {
    use ahash::HashSet;
    use mago_syntax::ast::*;
    use mago_syntax::walker::Walker;

    use crate::context::Context;
    use crate::heuristic::unused_parameter::utils::internal::FunctionCallWalker;

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
    pub struct ForeignVariable<'arena> {
        pub name: &'arena str,
        pub conditionally: bool,
    }

    pub fn is_super_global_variable(name: &str) -> bool {
        matches!(
            name,
            "$_GET" | "$_POST" | "$_COOKIE" | "$_REQUEST" | "$_SERVER" | "$_FILES" | "$_ENV" | "$_SESSION" | "$GLOBALS"
        )
    }

    pub fn is_predefined_variable(name: &str) -> bool {
        is_super_global_variable(name) || "$this" == name
    }

    pub fn potentially_contains_function_call<'ctx, 'ast, 'arena>(
        block: &'ast [Statement<'arena>],
        function_name: &'static str,
        context: &Context<'ctx, 'arena>,
    ) -> bool {
        let mut context = (false, context);

        let walker = FunctionCallWalker(function_name);
        for stmt in block {
            if context.0 {
                break;
            }

            walker.walk_statement(stmt, &mut context);
        }

        context.0
    }

    pub fn expression_potentially_contains_function_call<'ctx, 'ast, 'arena>(
        expression: &'ast Expression<'arena>,
        function_name: &'static str,
        context: &Context<'ctx, 'arena>,
    ) -> bool {
        let mut context = (false, context);

        FunctionCallWalker(function_name).walk_expression(expression, &mut context);

        context.0
    }

    pub fn is_variable_used_in_expression<'ctx, 'ast, 'arena>(
        expression: &'ast Expression<'arena>,
        context: &Context<'ctx, 'arena>,
        variable: &'arena str,
    ) -> bool {
        use crate::heuristic::unused_parameter::utils::internal::VariableReference;
        use crate::heuristic::unused_parameter::utils::internal::VariableWalker;

        let mut context = (Vec::default(), context, 0);

        VariableWalker.walk_expression(expression, &mut context);

        let variables = context.0;
        let mut reassigned = false;
        for variable_reference in variables {
            match variable_reference {
                VariableReference::Use(var) => {
                    if !reassigned && var == variable {
                        return true;
                    }
                }
                VariableReference::Assign(string_identifier, conditionally) => {
                    if !conditionally && string_identifier == variable {
                        reassigned = true;
                    }
                }
                VariableReference::Unset(string_identifier) => {
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

    pub fn get_foreign_variable_names<'arena>(
        stmts: &[Statement<'arena>],
        context: &Context<'_, 'arena>,
    ) -> Vec<ForeignVariable<'arena>> {
        use internal::*;

        let mut walker_context = (Vec::default(), context, 0);
        let walker = VariableWalker;
        for stmt in stmts {
            walker.walk_statement(stmt, &mut walker_context);
        }

        let variable_references = walker_context.0;
        let mut definitely_assigned = HashSet::default();
        let mut conditionally_assigned = HashSet::default();
        let mut foreign = Vec::default();
        let mut foreign_names = HashSet::default();

        for reference in variable_references {
            match reference {
                VariableReference::Use(name) => {
                    if !definitely_assigned.contains(&name) && !foreign_names.contains(&name) {
                        let is_conditional = conditionally_assigned.contains(&name);
                        foreign.push(ForeignVariable { name, conditionally: is_conditional });
                        foreign_names.insert(name);
                    }
                }
                VariableReference::Assign(name, is_conditional) => {
                    if is_conditional {
                        conditionally_assigned.insert(name);
                    } else {
                        definitely_assigned.insert(name);
                        conditionally_assigned.remove(&name);
                    }
                }
                VariableReference::Unset(name) => {
                    definitely_assigned.remove(&name);
                    conditionally_assigned.remove(&name);
                }
            }
        }

        foreign
    }

    mod internal {
        use super::is_predefined_variable;

        use mago_syntax::ast::*;
        use mago_syntax::walker::Walker;

        use crate::context::Context;

        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
        pub(super) enum VariableReference<'arena> {
            Use(&'arena str),
            Assign(&'arena str, bool),
            Unset(&'arena str),
        }

        #[derive(Debug)]
        pub(super) struct VariableWalker;

        #[derive(Debug)]
        pub(super) struct FunctionCallWalker(pub &'static str);

        impl<'ast, 'arena> Walker<'ast, 'arena, (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize)>
            for VariableWalker
        {
            fn walk_if(
                &self,
                r#if: &'ast If<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                self.walk_expression(r#if.condition, context);

                context.2 += 1;
                self.walk_if_body(&r#if.body, context);
                context.2 -= 1;
            }

            fn walk_for(
                &self,
                r#for: &'ast For<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                for i in r#for.initializations.iter() {
                    self.walk_expression(i, context);
                }

                for c in r#for.conditions.iter() {
                    self.walk_expression(c, context);
                }

                for i in r#for.increments.iter() {
                    self.walk_expression(i, context);
                }

                context.2 += 1;
                self.walk_for_body(&r#for.body, context);
                context.2 -= 1;
            }

            fn walk_while(
                &self,
                r#while: &'ast While<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                self.walk_expression(r#while.condition, context);
                context.2 += 1;
                self.walk_while_body(&r#while.body, context);
                context.2 -= 1;
            }

            fn walk_do_while(
                &self,
                do_while: &'ast DoWhile<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                context.2 += 1;
                self.walk_statement(do_while.statement, context);
                context.2 -= 1;
                self.walk_expression(do_while.condition, context);
            }

            fn walk_match_expression_arm(
                &self,
                match_expression_arm: &'ast MatchExpressionArm<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                for c in match_expression_arm.conditions.iter() {
                    self.walk_expression(c, context);
                }

                context.2 += 1;
                self.walk_expression(match_expression_arm.expression, context);
                context.2 -= 1;
            }

            fn walk_match_default_arm(
                &self,
                match_default_arm: &'ast MatchDefaultArm<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                context.2 += 1;
                self.walk_expression(match_default_arm.expression, context);
                context.2 -= 1;
            }

            fn walk_switch_expression_case(
                &self,
                switch_expression_case: &'ast SwitchExpressionCase<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                self.walk_expression(switch_expression_case.expression, context);
                context.2 += 1;
                for statement in switch_expression_case.statements.iter() {
                    self.walk_statement(statement, context);
                }
                context.2 -= 1;
            }

            fn walk_switch_default_case(
                &self,
                switch_default_case: &'ast SwitchDefaultCase<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                context.2 += 1;
                for statement in switch_default_case.statements.iter() {
                    self.walk_statement(statement, context);
                }
                context.2 -= 1;
            }

            fn walk_in_try_catch_clause(
                &self,
                try_catch_clause: &'ast TryCatchClause<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                if let Some(variable) = &try_catch_clause.variable {
                    context.0.push(VariableReference::Assign(variable.name, true));
                }

                context.2 += 1;
            }

            fn walk_out_try_catch_clause(
                &self,
                _try_catch_clause: &'ast TryCatchClause<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                context.2 -= 1;
            }

            fn walk_in_foreach_value_target(
                &self,
                foreach_value_target: &'ast ForeachValueTarget<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                scan_expression_for_assignment(foreach_value_target.value, &mut context.0, true);
            }

            fn walk_in_foreach_key_value_target(
                &self,
                foreach_key_value_target: &'ast ForeachKeyValueTarget<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                scan_expression_for_assignment(foreach_key_value_target.key, &mut context.0, true);
                scan_expression_for_assignment(foreach_key_value_target.value, &mut context.0, true);
            }

            fn walk_in_static_concrete_item(
                &self,
                static_concrete_item: &'ast StaticConcreteItem<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                context.0.push(VariableReference::Assign(static_concrete_item.variable.name, context.2 > 0));
            }

            fn walk_in_static_abstract_item(
                &self,
                static_abstract_item: &'ast StaticAbstractItem<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                context.0.push(VariableReference::Assign(static_abstract_item.variable.name, context.2 > 0));
            }

            fn walk_in_global(
                &self,
                global: &'ast Global<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                for variable in global.variables.iter() {
                    let Variable::Direct(variable) = variable else {
                        continue;
                    };
                    context.0.push(VariableReference::Assign(variable.name, context.2 > 0));
                }
            }

            fn walk_conditional(
                &self,
                conditional: &'ast Conditional<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                self.walk_expression(conditional.condition, context);

                context.2 += 1;
                if let Some(expr) = conditional.then {
                    self.walk_expression(expr, context);
                }
                self.walk_expression(conditional.r#else, context);
                context.2 -= 1;
            }

            fn walk_binary(
                &self,
                binary: &'ast Binary<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                self.walk_expression(binary.lhs, context);

                if !binary.operator.is_null_coalesce() && !binary.operator.is_logical() {
                    self.walk_expression(binary.rhs, context);
                    return;
                };

                context.2 += 1;
                self.walk_expression(binary.rhs, context);
                context.2 -= 1;
            }

            fn walk_assignment(
                &self,
                assignment: &'ast Assignment<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                self.walk_expression(assignment.rhs, context);

                let is_conditional = context.2 > 0;
                let mut variables = Vec::default();
                scan_expression_for_assignment(assignment.lhs, &mut variables, is_conditional);

                match assignment.operator {
                    AssignmentOperator::Assign(_) => {
                        context.0.extend(variables);
                    }
                    _ => {
                        for variable in variables {
                            if let VariableReference::Assign(name, is_cond) = variable {
                                context.0.push(VariableReference::Use(name));
                                context.0.push(VariableReference::Assign(name, is_cond));
                            } else {
                                context.0.push(variable);
                            }
                        }
                    }
                }

                self.walk_expression(assignment.lhs, context);
            }

            fn walk_in_direct_variable(
                &self,
                direct_variable: &'ast DirectVariable<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                if !is_predefined_variable(direct_variable.name) {
                    context.0.push(VariableReference::Use(direct_variable.name));
                }
            }

            fn walk_closure(
                &self,
                closure: &'ast Closure<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                if let Some(use_clause) = &closure.use_clause {
                    for use_clause_variable in use_clause.variables.iter() {
                        context.0.push(VariableReference::Use(use_clause_variable.variable.name));
                    }
                }
            }

            fn walk_in_arrow_function(
                &self,
                arrow_function: &'ast ArrowFunction<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                for parameter in arrow_function.parameter_list.parameters.iter() {
                    context.0.push(VariableReference::Assign(parameter.variable.name, false));
                }
            }

            fn walk_out_arrow_function(
                &self,
                arrow_function: &'ast ArrowFunction<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                for parameter in arrow_function.parameter_list.parameters.iter() {
                    context.0.push(VariableReference::Unset(parameter.variable.name));
                }
            }

            #[inline]
            fn walk_anonymous_class(
                &self,
                anonymous_class: &'ast AnonymousClass<'arena>,
                context: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
                if let Some(argument_list) = anonymous_class.argument_list.as_ref() {
                    self.walk_argument_list(argument_list, context);
                }
            }

            #[inline]
            fn walk_namespace(
                &self,
                _: &'ast Namespace<'arena>,
                _: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
            }

            #[inline]
            fn walk_class(
                &self,
                _: &'ast Class<'arena>,
                _: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
            }

            #[inline]
            fn walk_interface(
                &self,
                _: &'ast Interface<'arena>,
                _: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
            }

            #[inline]
            fn walk_trait(
                &self,
                _: &'ast Trait<'arena>,
                _: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
            }

            #[inline]
            fn walk_enum(
                &self,
                _: &'ast Enum<'arena>,
                _: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
            }

            #[inline]
            fn walk_function(
                &self,
                _: &'ast Function<'arena>,
                _: &mut (Vec<VariableReference<'arena>>, &Context<'_, 'arena>, usize),
            ) {
            }
        }

        impl<'ast, 'arena> Walker<'ast, 'arena, (bool, &Context<'_, 'arena>)> for FunctionCallWalker {
            fn walk_in_function_call(
                &self,
                function_call: &'ast FunctionCall<'arena>,
                context: &mut (bool, &Context<'_, 'arena>),
            ) {
                if context.0 {
                    return;
                }

                let Expression::Identifier(func) = function_call.function else {
                    return;
                };

                context.0 = context.1.resolve_function_name(func).eq_ignore_ascii_case(self.0);
            }

            #[inline]
            fn walk_closure(&self, _: &'ast Closure<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}

            #[inline]
            fn walk_arrow_function(&self, _: &'ast ArrowFunction<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}

            #[inline]
            fn walk_namespace(&self, _: &'ast Namespace<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}

            #[inline]
            fn walk_class(&self, _: &'ast Class<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}

            #[inline]
            fn walk_interface(&self, _: &'ast Interface<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}

            #[inline]
            fn walk_trait(&self, _: &'ast Trait<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}

            #[inline]
            fn walk_enum(&self, _: &'ast Enum<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}

            #[inline]
            fn walk_function(&self, _: &'ast Function<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}

            #[inline]
            fn walk_anonymous_class(&self, _: &'ast AnonymousClass<'arena>, _: &mut (bool, &Context<'_, 'arena>)) {}
        }

        fn scan_expression_for_assignment<'arena>(
            expression: &Expression<'arena>,
            variables: &mut Vec<VariableReference<'arena>>,
            is_conditional: bool,
        ) {
            match &expression {
                Expression::Variable(variable) => {
                    let Variable::Direct(variable) = variable else {
                        return;
                    };

                    if !is_predefined_variable(variable.name) {
                        variables.push(VariableReference::Assign(variable.name, is_conditional));
                    }
                }
                Expression::Array(array) => {
                    for element in array.elements.iter() {
                        match &element {
                            ArrayElement::KeyValue(key_value_array_element) => {
                                scan_expression_for_assignment(key_value_array_element.key, variables, is_conditional);
                                scan_expression_for_assignment(
                                    key_value_array_element.value,
                                    variables,
                                    is_conditional,
                                );
                            }
                            ArrayElement::Value(value_array_element) => {
                                scan_expression_for_assignment(value_array_element.value, variables, is_conditional);
                            }
                            _ => {}
                        }
                    }
                }
                Expression::LegacyArray(array) => {
                    for element in array.elements.iter() {
                        match &element {
                            ArrayElement::KeyValue(key_value_array_element) => {
                                scan_expression_for_assignment(key_value_array_element.key, variables, is_conditional);
                                scan_expression_for_assignment(
                                    key_value_array_element.value,
                                    variables,
                                    is_conditional,
                                );
                            }
                            ArrayElement::Value(value_array_element) => {
                                scan_expression_for_assignment(value_array_element.value, variables, is_conditional);
                            }
                            _ => {}
                        }
                    }
                }
                Expression::List(list) => {
                    for element in list.elements.iter() {
                        match &element {
                            ArrayElement::KeyValue(key_value_array_element) => {
                                scan_expression_for_assignment(key_value_array_element.key, variables, is_conditional);
                                scan_expression_for_assignment(
                                    key_value_array_element.value,
                                    variables,
                                    is_conditional,
                                );
                            }
                            ArrayElement::Value(value_array_element) => {
                                scan_expression_for_assignment(value_array_element.value, variables, is_conditional);
                            }
                            _ => {}
                        }
                    }
                }
                Expression::ArrayAppend(append) => {
                    if let Expression::Variable(Variable::Direct(variable)) = append.array {
                        let name = variable.name;
                        if !is_predefined_variable(name) {
                            variables.push(VariableReference::Use(variable.name));
                        }
                    }

                    scan_expression_for_assignment(append.array, variables, is_conditional);
                }
                Expression::ArrayAccess(access) => {
                    if let Expression::Variable(Variable::Direct(variable)) = access.array {
                        let name = variable.name;
                        if !is_predefined_variable(name) {
                            variables.push(VariableReference::Use(variable.name));
                        }
                    }

                    if let Expression::Variable(Variable::Direct(variable)) = access.index {
                        let name = variable.name;
                        if !is_predefined_variable(name) {
                            variables.push(VariableReference::Use(variable.name));
                        }
                    }

                    scan_expression_for_assignment(access.array, variables, is_conditional);
                    scan_expression_for_assignment(access.index, variables, is_conditional);
                }
                _ => {}
            }
        }
    }
}
