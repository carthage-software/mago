use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::Call;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Closure;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::FunctionLikeParameterList;
use mago_syntax::ast::MethodCall;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::PositionalArgument;
use mago_syntax::ast::StaticMethodCall;
use mago_syntax::ast::Variable;
use mago_text_edit::Safety;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::misc::get_single_return_statement;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferFirstClassCallableRule {
    meta: &'static RuleMeta,
    cfg: PreferFirstClassCallableConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferFirstClassCallableConfig {
    pub level: Level,
}

impl Default for PreferFirstClassCallableConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for PreferFirstClassCallableConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferFirstClassCallableRule {
    type Config = PreferFirstClassCallableConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer First Class Callable",
            code: "prefer-first-class-callable",
            description: indoc! {r"
                Promotes the use of first-class callable syntax (`...`) for creating closures.

                This rule identifies closures and arrow functions that do nothing but forward their arguments to another function or method.
                In such cases, the more concise and modern first-class callable syntax, introduced in PHP 8.1, can be used instead.
                This improves readability by reducing boilerplate code.
            "},
            good_example: indoc! {r"
                <?php

                $names = ['Alice', 'Bob', 'Charlie'];
                $uppercased_names = array_map(strtoupper(...), $names);
            "},
            bad_example: indoc! {r"
                <?php

                $names = ['Alice', 'Bob', 'Charlie'];
                $uppercased_names = array_map(fn($name) => strtoupper($name), $names);
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP81)),
        };
        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::ArrowFunction, NodeKind::Closure];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        if let Node::ArrowFunction(arrow_function) = node {
            let Expression::Call(call) = arrow_function.expression else {
                return;
            };

            if !is_call_forwarding(&arrow_function.parameter_list, call) {
                return;
            }

            if !is_convertible_to_first_class_callable(call) {
                return;
            }

            let span = arrow_function.span();

            let issue = Issue::new(
                self.cfg.level(),
                "Use first-class callable syntax `...` instead of a arrow function.",
            )
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message("This arrow function can be simplified to the `...` syntax."))
            .with_annotation(Annotation::secondary(arrow_function.parameter_list.span()).with_message("These parameters..."))
            .with_annotation(Annotation::secondary(call.get_argument_list().span()).with_message("...are directly forwarded here."))
            .with_note("This closure only forwards its arguments to another function or method, which can be expressed more concisely.")
            .with_help("Replace the arrow function with the first-class callable syntax (e.g., `strlen(...)`).");

            ctx.collector.propose(issue, |edits| {
                edits.push(TextEdit::delete(span.to_end(call.start_position())).with_safety(Safety::PotentiallyUnsafe));
                edits.push(
                    TextEdit::replace(call.get_argument_list().span(), "(...)").with_safety(Safety::PotentiallyUnsafe),
                );
            });
        }

        if let Node::Closure(closure) = node {
            let Some(return_stmt) = get_single_return_statement(&closure.body) else {
                return;
            };

            let Some(value) = &return_stmt.value else {
                return;
            };

            let Expression::Call(call) = value else {
                return;
            };

            if !is_call_forwarding(&closure.parameter_list, call) {
                return;
            }

            if !is_convertible_to_first_class_callable(call) {
                return;
            }

            if is_callee_reference_captured(call, closure) {
                return;
            }

            let issue = Issue::new(
                self.cfg.level(),
                "Use first-class callable syntax `...` instead of a closure.",
            )
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(node.span()).with_message("This closure can be simplified to the `...` syntax."))
            .with_annotation(Annotation::secondary(closure.parameter_list.span()).with_message("These parameters..."))
            .with_annotation(Annotation::secondary(call.get_argument_list().span()).with_message("...are directly forwarded here."))
            .with_note("This closure only forwards its arguments to another function or method, which can be expressed more concisely.")
            .with_help("Replace the closure with the first-class callable syntax (e.g., `strlen(...)`).");

            ctx.collector.propose(issue, |edits| {
                let closure_end = closure.end_position();

                edits.push(
                    TextEdit::delete(closure.span().to_end(value.start_position()))
                        .with_safety(Safety::PotentiallyUnsafe),
                );
                edits.push(
                    TextEdit::delete(return_stmt.terminator.span().to_end(closure_end))
                        .with_safety(Safety::PotentiallyUnsafe),
                );
                edits.push(
                    TextEdit::replace(call.get_argument_list().span(), "(...)").with_safety(Safety::PotentiallyUnsafe),
                );
            });
        }
    }
}

pub(super) fn is_call_forwarding<'ast, 'arena>(
    parameter_list: &'ast FunctionLikeParameterList<'arena>,
    call: &'ast Call<'arena>,
) -> bool {
    let argument_list = call.get_argument_list();

    if parameter_list.parameters.len() != argument_list.arguments.len() {
        return false;
    }

    for (idx, parameter) in parameter_list.parameters.iter().enumerate() {
        let Some(argument) = argument_list.arguments.get(idx) else {
            return false;
        };

        let Argument::Positional(PositionalArgument { value, .. }) = argument else {
            return false;
        };

        let Expression::Variable(Variable::Direct(direct_variable)) = value else {
            return false;
        };

        if direct_variable.name != parameter.variable.name {
            return false;
        }
    }

    // Same number of parameters and arguments, and all arguments are direct references to the corresponding parameters
    // -> it's a call forwarding
    true
}

pub(super) fn is_convertible_to_first_class_callable<'ast, 'arena>(call: &'ast Call<'arena>) -> bool {
    matches!(
        call,
        Call::Function(FunctionCall { function: Expression::Identifier(_) | Expression::Variable(_), .. })
            | Call::Method(MethodCall {
                object: Expression::Variable(_),
                method: ClassLikeMemberSelector::Identifier(_),
                ..
            })
            | Call::StaticMethod(StaticMethodCall {
                class: Expression::Identifier(_) | Expression::Self_(_) | Expression::Static(_) | Expression::Parent(_),
                method: ClassLikeMemberSelector::Identifier(_),
                ..
            })
    )
}

fn is_callee_reference_captured<'ast, 'arena>(call: &'ast Call<'arena>, closure: &'ast Closure<'arena>) -> bool {
    let callee_var_name = match call {
        Call::Function(FunctionCall { function: Expression::Variable(Variable::Direct(var)), .. }) => var.name,
        Call::Method(MethodCall { object: Expression::Variable(Variable::Direct(var)), .. }) => var.name,
        _ => return false,
    };

    closure.use_clause.as_ref().is_some_and(|use_clause| {
        use_clause.variables.iter().any(|v| v.ampersand.is_some() && v.variable.name == callee_var_name)
    })
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferFirstClassCallableRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = method_on_function_result_with_no_arguments,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            function getSomeClass() { return new SomeClass(); }

            run(fn() => getSomeClass()->method());
        "#}
    }

    test_lint_success! {
        name = method_on_function_result,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            function getSomeClass() { return new SomeClass(); }

            run(fn($x) => getSomeClass()->method($x));
        "#}
    }

    test_lint_success! {
        name = null_safe_method_call,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            $someClass = new SomeClass();

            run(fn($x) => $someClass?->method($x));
        "#}
    }

    test_lint_success! {
        name = dynamic_method_name,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            $someClass = new SomeClass();
            $method = "method";

            run(fn($x) => $someClass->$method($x));
        "#}
    }

    test_lint_success! {
        name = closure_method_on_function_result,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            function getSomeClass() { return new SomeClass(); }

            run(function($x) { return getSomeClass()->method($x); });
        "#}
    }

    test_lint_success! {
        name = closure_with_reference_captured_callee,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            $callable = "strlen";

            run(function($x) use (&$callable) { return $callable($x); });
        "#}
    }

    test_lint_success! {
        name = closure_with_reference_captured_method_object,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            $obj = new SomeClass();

            run(function($x) use (&$obj) { return $obj->method($x); });
        "#}
    }

    test_lint_failure! {
        name = simple_function_call,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            run(fn($x) => strlen($x));
        "#}
    }

    test_lint_failure! {
        name = simple_method_call,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            run(fn($x) => $this->method($x));
        "#}
    }

    test_lint_failure! {
        name = simple_static_method_call,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            run(fn($x) => SomeClass::method($x));
        "#}
    }

    test_lint_failure! {
        name = closure_with_reference_capture_on_non_callee,
        rule = PreferFirstClassCallableRule,
        code = indoc! {r#"
            <?php

            $unused = null;

            run(function($x) use (&$unused) { return strlen($x); });
        "#}
    }
}
