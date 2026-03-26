use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_docblock::document::TagKind;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::FunctionLikeParameter;
use mago_syntax::ast::Hint;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Program;
use mago_syntax::ast::Statement;
use mago_syntax::ast::TriviaKind;
use mago_syntax::comments::docblock::get_docblock_for_node;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoUnnecessaryDocblockTagRule {
    meta: &'static RuleMeta,
    cfg: NoUnnecessaryDocblockTagConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoUnnecessaryDocblockTagConfig {
    pub level: Level,
}

impl Default for NoUnnecessaryDocblockTagConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoUnnecessaryDocblockTagConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

fn hint_to_string(hint: &Hint<'_>) -> String {
    match hint {
        Hint::Identifier(id) => id.value().trim_start_matches('\\').to_string(),
        Hint::Nullable(n) => format!("?{}", hint_to_string(n.hint)),
        Hint::Union(u) => format!("{}|{}", hint_to_string(u.left), hint_to_string(u.right)),
        Hint::Intersection(i) => format!("{}&{}", hint_to_string(i.left), hint_to_string(i.right)),
        Hint::Parenthesized(p) => format!("({})", hint_to_string(p.hint)),
        Hint::Null(_) => "null".to_string(),
        Hint::True(_) => "true".to_string(),
        Hint::False(_) => "false".to_string(),
        Hint::Array(_) => "array".to_string(),
        Hint::Callable(_) => "callable".to_string(),
        Hint::Static(_) => "static".to_string(),
        Hint::Self_(_) => "self".to_string(),
        Hint::Parent(_) => "parent".to_string(),
        Hint::Void(id)
        | Hint::Never(id)
        | Hint::Float(id)
        | Hint::Bool(id)
        | Hint::Integer(id)
        | Hint::String(id)
        | Hint::Object(id)
        | Hint::Mixed(id)
        | Hint::Iterable(id) => id.value.to_string(),
    }
}

fn is_param_tag_unnecessary(description: &str, param: &FunctionLikeParameter<'_>) -> bool {
    let Some(hint) = param.hint.as_ref() else {
        return false;
    };

    let description = description.trim();
    if description.is_empty() {
        return false;
    }

    let native_type = hint_to_string(hint);
    let param_name = param.variable.name;

    let Some(dollar_pos) = description.find(param_name) else {
        return false;
    };

    let type_part = description[..dollar_pos].trim();
    let after_param = description[dollar_pos + param_name.len()..].trim();

    if !after_param.is_empty() {
        return false;
    }

    type_part.eq_ignore_ascii_case(&native_type)
}

fn is_return_tag_unnecessary(description: &str, return_hint: &Hint<'_>) -> bool {
    let description = description.trim();
    if description.is_empty() {
        return false;
    }

    let native_type = hint_to_string(return_hint);

    let (type_part, rest) = match description.find(char::is_whitespace) {
        Some(pos) => (&description[..pos], description[pos..].trim()),
        None => (description, ""),
    };

    if !rest.is_empty() {
        return false;
    }

    type_part.eq_ignore_ascii_case(&native_type)
}

impl LintRule for NoUnnecessaryDocblockTagRule {
    type Config = NoUnnecessaryDocblockTagConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unnecessary Docblock Tag",
            code: "no-unnecessary-docblock-tag",
            description: indoc! {"
                Detects `@param` and `@return` docblock tags that merely repeat the type
                already present in the function signature without adding any description.

                These tags add visual noise without providing additional information.
                Docblock tags are useful when they provide more specific types (e.g.,
                `list<string>` for an `array` parameter) or include a description.
            "},
            good_example: indoc! {r#"
                <?php

                /**
                 * Transforms the given value.
                 *
                 * @param string $value The value to transform
                 * @return string The transformed result
                 */
                function transform(string $value): string {
                    return strtoupper($value);
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                /**
                 * @param string $value
                 * @return string
                 */
                function identity(string $value): string {
                    return $value;
                }
            "#},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        for stmt in &program.statements {
            self.check_statement(ctx, program, stmt);
        }
    }
}

impl NoUnnecessaryDocblockTagRule {
    fn check_statement<'a, 'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        program: &'a Program<'arena>,
        stmt: &'a Statement<'arena>,
    ) {
        match stmt {
            Statement::Function(func) => {
                self.check_function_docblock(
                    ctx,
                    program,
                    func,
                    &func.parameter_list.parameters.nodes,
                    func.return_type_hint.as_ref().map(|r| &r.hint),
                );
            }
            Statement::Namespace(ns) => {
                for inner_stmt in ns.statements() {
                    self.check_statement(ctx, program, inner_stmt);
                }
            }
            Statement::Class(class) => {
                self.check_methods(ctx, program, class.members.iter());
            }
            Statement::Interface(iface) => {
                self.check_methods(ctx, program, iface.members.iter());
            }
            Statement::Trait(tr) => {
                self.check_methods(ctx, program, tr.members.iter());
            }
            Statement::Enum(en) => {
                self.check_methods(ctx, program, en.members.iter());
            }
            _ => {}
        }
    }

    fn check_methods<'a, 'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        program: &'a Program<'arena>,
        members: impl Iterator<Item = &'a ClassLikeMember<'arena>>,
    ) {
        for member in members {
            if let ClassLikeMember::Method(method) = member {
                self.check_function_docblock(
                    ctx,
                    program,
                    method,
                    &method.parameter_list.parameters.nodes,
                    method.return_type_hint.as_ref().map(|r| &r.hint),
                );
            }
        }
    }

    fn check_function_docblock<'a, 'arena>(
        &self,
        ctx: &mut LintContext<'_, 'arena>,
        program: &'a Program<'arena>,
        node: &'a impl HasSpan,
        params: &'a [FunctionLikeParameter<'arena>],
        return_hint: Option<&'a Hint<'arena>>,
    ) {
        let Some(trivia) = get_docblock_for_node(program, ctx.source_file, node) else {
            return;
        };

        if trivia.kind != TriviaKind::DocBlockComment {
            return;
        }

        let Ok(document) = mago_docblock::parse_trivia(ctx.arena, trivia) else {
            return;
        };

        for tag in document.get_tags() {
            match tag.kind {
                TagKind::Param => {
                    for param in params {
                        if is_param_tag_unnecessary(tag.description, param) {
                            ctx.collector.report(
                                Issue::new(self.cfg.level, "Unnecessary `@param` tag.")
                                    .with_code(self.meta.code)
                                    .with_annotation(
                                        Annotation::primary(tag.span)
                                            .with_message("This `@param` tag just repeats the native type hint"),
                                    )
                                    .with_help("Remove this tag or add a description that provides additional context."),
                            );
                            break;
                        }
                    }
                }
                TagKind::Return => {
                    if let Some(return_hint) = return_hint && is_return_tag_unnecessary(tag.description, return_hint) {
                        ctx.collector.report(
                            Issue::new(self.cfg.level, "Unnecessary `@return` tag.")
                                .with_code(self.meta.code)
                                .with_annotation(
                                    Annotation::primary(tag.span)
                                        .with_message("This `@return` tag just repeats the native return type"),
                                )
                                .with_help("Remove this tag or add a description that provides additional context."),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoUnnecessaryDocblockTagRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = no_docblock_at_all,
        rule = NoUnnecessaryDocblockTagRule,
        code = indoc! {r#"
            <?php

            function identity(string $value): string {
                return $value;
            }
        "#}
    }

    test_lint_success! {
        name = param_with_description,
        rule = NoUnnecessaryDocblockTagRule,
        code = indoc! {r#"
            <?php

            /**
             * @param string $value The value to transform
             */
            function foo(string $value): void {
            }
        "#}
    }

    test_lint_success! {
        name = return_with_description,
        rule = NoUnnecessaryDocblockTagRule,
        code = indoc! {r#"
            <?php

            /**
             * @return string The transformed result
             */
            function foo(): string {
                return 'hello';
            }
        "#}
    }

    test_lint_success! {
        name = more_specific_type_than_native,
        rule = NoUnnecessaryDocblockTagRule,
        code = indoc! {r#"
            <?php

            /**
             * @param list<string> $items
             */
            function foo(array $items): void {
            }
        "#}
    }

    test_lint_success! {
        name = function_without_type_hints,
        rule = NoUnnecessaryDocblockTagRule,
        code = indoc! {r#"
            <?php

            /**
             * @param string $value
             */
            function foo($value) {
            }
        "#}
    }

    test_lint_failure! {
        name = redundant_param_tag,
        rule = NoUnnecessaryDocblockTagRule,
        code = indoc! {r#"
            <?php

            /**
             * @param string $value
             */
            function foo(string $value): void {
            }
        "#}
    }

    test_lint_failure! {
        name = redundant_return_tag,
        rule = NoUnnecessaryDocblockTagRule,
        code = indoc! {r#"
            <?php

            /**
             * @return string
             */
            function foo(): string {
                return 'hello';
            }
        "#}
    }

    test_lint_failure! {
        name = both_param_and_return_redundant,
        rule = NoUnnecessaryDocblockTagRule,
        count = 2,
        code = indoc! {r#"
            <?php

            /**
             * @param string $value
             * @return string
             */
            function identity(string $value): string {
                return $value;
            }
        "#}
    }
}
