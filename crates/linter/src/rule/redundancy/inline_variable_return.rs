use indoc::indoc;
use mago_syntax::ast::UnaryPrefix;
use mago_syntax::ast::UnaryPrefixOperator;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Closure;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::Variable;
use mago_syntax::walker::MutWalker;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

/// Rule that detects unnecessary variable assignment immediately before return.
///
/// This pattern:
/// ```php
/// $x = getValue();
/// return $x;
/// ```
///
/// Can be simplified to:
/// ```php
/// return getValue();
/// ```
#[derive(Debug, Clone)]
pub struct InlineVariableReturnRule {
    meta: &'static RuleMeta,
    cfg: InlineVariableReturnConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct InlineVariableReturnConfig {
    pub level: Level,
}

impl Default for InlineVariableReturnConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for InlineVariableReturnConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for InlineVariableReturnRule {
    type Config = InlineVariableReturnConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Inline Variable Return",
            code: "inline-variable-return",
            description: indoc! {"
                Detects unnecessary variable assignments immediately before returning the variable.

                When a variable is only used once right after being assigned, the assignment
                can be inlined into the return statement.
            "},
            good_example: indoc! {r"
                <?php

                function getValue() {
                    return computeResult();
                }

                function process() {
                    $result = computeResult();
                    log($result);
                    return $result;
                }
            "},
            bad_example: indoc! {r"
                <?php

                function getValue() {
                    $result = computeResult();
                    return $result;
                }

                function getArray() {
                    $arr = [1, 2, 3];
                    return $arr;
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Program,
            NodeKind::Block,
            NodeKind::Namespace,
            NodeKind::DeclareColonDelimitedBody,
            NodeKind::ForeachColonDelimitedBody,
            NodeKind::WhileColonDelimitedBody,
            NodeKind::ForColonDelimitedBody,
            NodeKind::IfColonDelimitedBody,
            NodeKind::IfColonDelimitedBodyElseIfClause,
            NodeKind::IfColonDelimitedBodyElseClause,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let statements = match node {
            Node::Program(program) => program.statements.as_slice(),
            Node::Block(block) => block.statements.as_slice(),
            Node::Namespace(namespace) => namespace.statements().as_slice(),
            Node::DeclareColonDelimitedBody(body) => body.statements.as_slice(),
            Node::ForeachColonDelimitedBody(body) => body.statements.as_slice(),
            Node::WhileColonDelimitedBody(body) => body.statements.as_slice(),
            Node::ForColonDelimitedBody(body) => body.statements.as_slice(),
            Node::IfColonDelimitedBody(body) => body.statements.as_slice(),
            Node::IfColonDelimitedBodyElseIfClause(body) => body.statements.as_slice(),
            Node::IfColonDelimitedBodyElseClause(body) => body.statements.as_slice(),
            _ => return,
        };

        for window in statements.windows(2) {
            let [Statement::Expression(expr_stmt), Statement::Return(return_stmt)] = window else {
                continue;
            };

            let Expression::Assignment(assignment) = expr_stmt.expression else {
                continue;
            };

            if !assignment.operator.is_assign() {
                continue;
            }

            let Expression::Variable(Variable::Direct(assigned_var)) = assignment.lhs else {
                continue;
            };

            let Some(return_value) = &return_stmt.value else {
                continue;
            };

            let Expression::Variable(Variable::Direct(return_var)) = return_value else {
                continue;
            };

            if assigned_var.name != return_var.name {
                continue;
            }

            if involves_references(assignment.rhs, assigned_var.name) {
                continue;
            }

            let rhs_text = &ctx.source_file.contents[assignment.rhs.span().to_range_usize()];

            let issue = Issue::new(self.cfg.level(), "Variable assignment can be inlined into the return statement.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(expr_stmt.span()).with_message("This assignment is immediately returned"),
                )
                .with_annotation(
                    Annotation::secondary(return_stmt.span()).with_message("The variable is only used here"),
                )
                .with_note("Assigning to a variable just to return it immediately is redundant.")
                .with_help(format!("Return the expression directly: `return {rhs_text};`"));

            ctx.collector.propose(issue, |edits| {
                let assign_span = expr_stmt.span();
                let delete_end =
                    find_next_non_whitespace(ctx.source_file.contents.as_bytes(), assign_span.end_offset());
                let delete_range = assign_span.start_offset()..delete_end;

                edits.push(TextEdit::delete(delete_range));

                let return_var_span = return_var.span;
                edits.push(TextEdit::replace(return_var_span, rhs_text));
            });
        }
    }
}

/// Check if the expression is by reference,
/// or has a reference capture of the given variable name.
fn involves_references(expr: &Expression<'_>, var_name: &str) -> bool {
    if let Expression::Parenthesized(parenthesized) = expr {
        return involves_references(parenthesized.expression, var_name);
    }

    if let Expression::UnaryPrefix(UnaryPrefix { operator: UnaryPrefixOperator::Reference(_), .. }) = expr {
        return true;
    }

    struct RefCaptureChecker<'a> {
        var_name: &'a str,
        found: bool,
    }

    impl<'arena> MutWalker<'_, 'arena, ()> for RefCaptureChecker<'_> {
        fn walk_closure(&mut self, closure: &'_ Closure<'arena>, _: &mut ()) {
            let Some(use_clause) = &closure.use_clause else {
                return;
            };

            if use_clause.variables.iter().any(|v| v.ampersand.is_some() && v.variable.name == self.var_name) {
                self.found = true;
            }
        }
    }

    let mut checker = RefCaptureChecker { var_name, found: false };
    checker.walk_expression(expr, &mut ());
    checker.found
}

/// Find the next non-whitespace character position after the given offset.
fn find_next_non_whitespace(bytes: &[u8], offset: u32) -> u32 {
    let mut pos = offset as usize;
    while pos < bytes.len() {
        match bytes[pos] {
            b' ' | b'\t' | b'\r' | b'\n' => pos += 1,
            _ => break,
        }
    }
    pos as u32
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::InlineVariableReturnRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_variable_used_multiple_times,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = computeResult();
                log($result);
                return $result;
            }
        "}
    }

    test_lint_success! {
        name = valid_different_variables,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = computeResult();
                return $other;
            }
        "}
    }

    test_lint_success! {
        name = valid_complex_return_expression,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = computeResult();
                return $result + 1;
            }
        "}
    }

    test_lint_success! {
        name = valid_not_simple_assignment,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result += 1;
                return $result;
            }
        "}
    }

    test_lint_success! {
        name = valid_return_without_value,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = computeResult();
                return;
            }
        "}
    }

    test_lint_success! {
        name = valid_not_consecutive,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = computeResult();
                doSomething();
                return $result;
            }
        "}
    }

    test_lint_success! {
        name = valid_array_destructuring,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                [$a, $b] = getValues();
                return $a;
            }
        "}
    }

    test_lint_success! {
        name = valid_closure_with_reference_capture,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = function() use (&$result) { return $result; };
                return $result;
            }
        "}
    }

    test_lint_success! {
        name = valid_parenthesized_closure_with_reference_capture,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = (function() use (&$result) { return $result; });
                return $result;
            }
        "}
    }

    test_lint_success! {
        name = valid_by_ref,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function &get_gategory_by_path(string $path): ?array {
                return &find_node();
            }
        "}
    }

    test_lint_failure! {
        name = simple_function_call,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = computeResult();
                return $result;
            }
        "}
    }

    test_lint_failure! {
        name = array_literal,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getArray() {
                $arr = [1, 2, 3];
                return $arr;
            }
        "}
    }

    test_lint_failure! {
        name = new_expression,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getInstance() {
                $instance = new MyClass();
                return $instance;
            }
        "}
    }

    test_lint_failure! {
        name = arithmetic_expression,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function calculate() {
                $sum = $a + $b;
                return $sum;
            }
        "}
    }

    test_lint_failure! {
        name = string_literal,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getMessage() {
                $message = 'Hello, world!';
                return $message;
            }
        "}
    }

    test_lint_failure! {
        name = method_chain,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getResult() {
                $result = $this->service->process()->getData();
                return $result;
            }
        "}
    }

    test_lint_failure! {
        name = closure_with_value_capture,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = function() use ($result) { return $result; };
                return $result;
            }
        "}
    }

    test_lint_failure! {
        name = closure_with_different_reference_capture,
        rule = InlineVariableReturnRule,
        code = indoc! {r"
            <?php

            function getValue() {
                $result = function() use (&$other) { return $other; };
                return $result;
            }
        "}
    }
}
