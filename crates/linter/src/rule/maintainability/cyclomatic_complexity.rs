use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::misc::is_method_setter_or_getter;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct CyclomaticComplexityRule {
    meta: &'static RuleMeta,
    cfg: CyclomaticComplexityConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct CyclomaticComplexityConfig {
    pub level: Level,
    pub threshold: usize,
}

impl Default for CyclomaticComplexityConfig {
    fn default() -> Self {
        Self { level: Level::Error, threshold: 15 }
    }
}

impl Config for CyclomaticComplexityConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for CyclomaticComplexityRule {
    type Config = CyclomaticComplexityConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Cyclomatic Complexity",
            code: "cyclomatic-complexity",
            description: indoc! {r#"
                Checks the cyclomatic complexity of classes, traits, enums, interfaces, functions, and closures.

                Cyclomatic complexity is a measure of the number of linearly independent paths through a program's source code.
            "#},
            good_example: indoc! {r#"
                <?php

                function validateUser($user) {
                    if (!isValidEmail($user['email'])) {
                        return false;
                    }

                    if (!isValidAge($user['age'])) {
                        return false;
                    }

                    if (!hasRequiredFields($user)) {
                        return false;
                    }

                    return true;
                }

                function isValidEmail($email) {
                    return filter_var($email, FILTER_VALIDATE_EMAIL) !== false;
                }

                function isValidAge($age) {
                    return $age >= 18 && $age <= 120;
                }

                function hasRequiredFields($user) {
                    return isset($user['name']) && isset($user['email']);
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                function validateUser($user) {
                    if (!isset($user['email'])) {
                        return false;
                    }

                    if (!filter_var($user['email'], FILTER_VALIDATE_EMAIL)) {
                        return false;
                    }

                    if (!isset($user['age'])) {
                        return false;
                    }

                    if ($user['age'] < 18) {
                        return false;
                    }

                    if ($user['age'] > 120) {
                        return false;
                    }

                    if (!isset($user['name'])) {
                        return false;
                    }

                    if (strlen($user['name']) < 2) {
                        return false;
                    }

                    if (!isset($user['country'])) {
                        return false;
                    }

                    if (!in_array($user['country'], ['US', 'UK', 'CA'])) {
                        return false;
                    }

                    if (isset($user['phone'])) {
                        if (!preg_match('/^\d{10}$/', $user['phone'])) {
                            return false;
                        }
                    }

                    if (isset($user['preferences'])) {
                        if (is_array($user['preferences'])) {
                            foreach ($user['preferences'] as $key => $value) {
                                if ($key === 'newsletter') {
                                    if ($value !== true && $value !== false) {
                                        return false;
                                    }
                                }
                            }
                        }
                    }

                    if (isset($user['address'])) {
                        if (!isset($user['address']['street'])) {
                            return false;
                        }
                        if (!isset($user['address']['city'])) {
                            return false;
                        }
                    }

                    return true;
                }
            "#},
            category: Category::Maintainability,
            requirements: RuleRequirements::None,
        };
        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Class,
            NodeKind::Trait,
            NodeKind::AnonymousClass,
            NodeKind::Enum,
            NodeKind::Interface,
            NodeKind::Function,
            NodeKind::Closure,
        ];
        TARGETS
    }

    fn build(settings: &RuleSettings<CyclomaticComplexityConfig>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        match node {
            Node::Class(n) => self.check_class_like("Class", n.members.as_slice(), n.span(), ctx),
            Node::Trait(n) => self.check_class_like("Trait", n.members.as_slice(), n.span(), ctx),
            Node::AnonymousClass(n) => self.check_class_like("Class", n.members.as_slice(), n.span(), ctx),
            Node::Enum(n) => self.check_class_like("Enum", n.members.as_slice(), n.span(), ctx),
            Node::Interface(n) => self.check_class_like("Interface", n.members.as_slice(), n.span(), ctx),
            Node::Function(n) => self.check_function_like("Function", &n.body, n.span(), ctx),
            Node::Closure(n) => self.check_function_like("Closure", &n.body, n.span(), ctx),
            _ => (),
        }
    }
}

impl CyclomaticComplexityRule {
    fn check_class_like<'arena>(
        &self,
        kind: &'static str,
        members: &[ClassLikeMember<'arena>],
        span: impl HasSpan,
        ctx: &mut LintContext<'_, 'arena>,
    ) {
        let threshold = self.cfg.threshold;

        let complexity = get_cyclomatic_complexity_of_class_like_members(members);
        if complexity > threshold {
            let issue = Issue::new(self.cfg.level, format!("{kind} has high complexity."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(span.span()).with_message(format!(
                    "{kind} has a cyclomatic complexity of {complexity}, which exceeds the threshold of {threshold}."
                )));

            ctx.collector.report(issue);
        }
    }

    fn check_function_like<'arena>(
        &self,
        kind: &'static str,
        body: &Block<'arena>,
        span: impl HasSpan,
        ctx: &mut LintContext<'_, 'arena>,
    ) {
        let threshold = self.cfg.threshold;

        let complexity = get_cyclomatic_complexity_of_node(Node::Block(body));

        if complexity > threshold {
            let issue = Issue::new(self.cfg.level, format!("{kind} has high complexity."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(span.span()).with_message(format!(
                    "{kind} has a cyclomatic complexity of {complexity}, which exceeds the threshold of {threshold}."
                )));

            ctx.collector.report(issue);
        }
    }
}

#[inline]
fn get_cyclomatic_complexity_of_class_like_members<'ast, 'arena>(
    class_like_members: &'ast [ClassLikeMember<'arena>],
) -> usize {
    let mut cyclomatic_complexity = 0;
    for member in class_like_members {
        let ClassLikeMember::Method(method) = member else {
            continue;
        };

        let Some(method_cyclomatic_complexity) = get_cyclomatic_complexity_of_method(method) else {
            continue;
        };

        cyclomatic_complexity += method_cyclomatic_complexity - 1;
    }

    cyclomatic_complexity
}

#[inline]
fn get_cyclomatic_complexity_of_method<'ast, 'arena>(method: &'ast Method<'arena>) -> Option<usize> {
    if is_method_setter_or_getter(method) {
        return None;
    }

    Some(if method.is_abstract() { 1 } else { get_cyclomatic_complexity_of_node(Node::Method(method)) + 1 })
}

#[inline]
fn get_cyclomatic_complexity_of_node<'ast, 'arena>(node: Node<'ast, 'arena>) -> usize {
    let mut number = 0;

    for child in node.children() {
        number += get_cyclomatic_complexity_of_node(child);
    }

    match node {
        Node::If(_)
        | Node::IfStatementBodyElseIfClause(_)
        | Node::IfColonDelimitedBodyElseIfClause(_)
        | Node::For(_)
        | Node::Foreach(_)
        | Node::While(_)
        | Node::DoWhile(_)
        | Node::TryCatchClause(_)
        | Node::Conditional(_) => number += 1,
        Node::Binary(operation) => match operation.operator {
            operator if operator.is_logical() || operator.is_null_coalesce() => number += 1,
            BinaryOperator::Spaceship(_) => number += 2,
            _ => (),
        },
        Node::SwitchCase(case) if case.is_default() => {
            number += 1;
        }
        _ => (),
    }

    number
}
