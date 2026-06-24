use indoc::indoc;
use mago_allocator::Arena;
use mago_phpdoc_syntax::PHPDocParser;
use mago_phpdoc_syntax::cst::AssertPattern;
use mago_phpdoc_syntax::cst::TagValue;
use mago_phpdoc_syntax::cst::TemplateTagValue;
use mago_phpdoc_syntax::cst::r#type::GenericParameters;
use mago_phpdoc_syntax::cst::r#type::SingleGenericParameter;
use mago_phpdoc_syntax::cst::r#type::Type;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::TriviaKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoShorthandArrayTypeRule {
    meta: &'static RuleMeta,
    cfg: NoShorthandArrayTypeConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct NoShorthandArrayTypeConfig {
    pub level: Level,
    pub check_generic_array: bool,
}

impl Default for NoShorthandArrayTypeConfig {
    fn default() -> Self {
        Self { level: Level::Warning, check_generic_array: false }
    }
}

impl Config for NoShorthandArrayTypeConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

impl LintRule for NoShorthandArrayTypeRule {
    type Config = NoShorthandArrayTypeConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Shorthand Array Type",
            code: "no-shorthand-array-type",
            description: indoc! {"
                Detects legacy `T[]` array shorthand in docblock type expressions.

                The shorthand means `array<array-key, T>`, but it does not communicate whether
                the type is intended to be a sequential list or a keyed array.
            "},
            good_example: indoc! {r"
                <?php

                /**
                 * @param list<User> $users
                 *
                 * @return array<array-key, User>
                 */
                function get_users(array $users): array
                {
                    return $users;
                }
            "},
            bad_example: indoc! {r"
                <?php

                /**
                 * @param User[] $users
                 *
                 * @return User[]
                 */
                function get_users(array $users): array
                {
                    return $users;
                }
            "},
            category: Category::Consistency,
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

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Program(program) = node else {
            return;
        };

        for trivia in &program.trivia {
            if trivia.kind != TriviaKind::DocBlockComment {
                continue;
            }

            let document = PHPDocParser::parse_with_span(ctx.arena, trivia.value, trivia.span);
            for tag in document.tags() {
                self.check_tag_value(ctx, &tag.value);
            }
        }
    }
}

impl NoShorthandArrayTypeRule {
    fn check_tag_value<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, value: &TagValue<'arena>)
    where
        A: Arena,
    {
        match value {
            TagValue::Param(value) => self.check_type(ctx, value.r#type),
            TagValue::ParamOut(value) => self.check_type(ctx, value.r#type),
            TagValue::ParamClosureThis(value) => self.check_type(ctx, value.r#type),
            TagValue::Return(value) | TagValue::RealReturn(value) => self.check_type(ctx, value.r#type),
            TagValue::Var(value) => self.check_type(ctx, value.r#type),
            TagValue::Throws(value) => self.check_type(ctx, value.r#type),
            TagValue::Mixin(value) => self.check_type(ctx, value.r#type),
            TagValue::SelfOut(value) => self.check_type(ctx, value.r#type),
            TagValue::Template(value) => self.check_template_tag_value(ctx, value),
            TagValue::Extends(value) => self.check_type(ctx, value.r#type),
            TagValue::Implements(value) => self.check_type(ctx, value.r#type),
            TagValue::Use(value) => self.check_type(ctx, value.r#type),
            TagValue::RequireExtends(value) => self.check_type(ctx, value.r#type),
            TagValue::RequireImplements(value) => self.check_type(ctx, value.r#type),
            TagValue::Sealed(value) => self.check_type(ctx, value.r#type),
            TagValue::Inheritors(value) => self.check_type(ctx, value.r#type),
            TagValue::Method(value) => {
                if let Some(return_type) = value.return_type {
                    self.check_type(ctx, return_type);
                }

                if let Some(templates) = value.templates {
                    for template in templates.entries.iter() {
                        self.check_template_tag_value(ctx, &template.template);
                    }
                }

                for parameter in value.parameters.entries.iter() {
                    if let Some(parameter_type) = &parameter.r#type {
                        self.check_type(ctx, parameter_type);
                    }
                }
            }
            TagValue::Property(value) | TagValue::PropertyRead(value) | TagValue::PropertyWrite(value) => {
                if let Some(property_type) = value.r#type {
                    self.check_type(ctx, property_type);
                }
            }
            TagValue::Assert(value) | TagValue::AssertIfTrue(value) | TagValue::AssertIfFalse(value) => {
                if let AssertPattern::Type(asserted_type) = value.pattern {
                    self.check_type(ctx, asserted_type);
                }
            }
            TagValue::Where(value) => self.check_type(ctx, value.r#type),
            TagValue::TypeAlias(value) => self.check_type(ctx, value.r#type),
            _ => {}
        }
    }

    fn check_template_tag_value<'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        value: &TemplateTagValue<'arena>,
    ) where
        A: Arena,
    {
        if let Some(bound) = &value.bound {
            self.check_type(ctx, bound.r#type);
        }

        if let Some(default) = &value.default {
            self.check_type(ctx, default.r#type);
        }
    }

    fn check_type<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, r#type: &Type<'arena>)
    where
        A: Arena,
    {
        match r#type {
            Type::Parenthesized(r#type) => self.check_type(ctx, r#type.inner),
            Type::Union(r#type) => {
                self.check_type(ctx, r#type.left);
                self.check_type(ctx, r#type.right);
            }
            Type::Intersection(r#type) => {
                self.check_type(ctx, r#type.left);
                self.check_type(ctx, r#type.right);
            }
            Type::Nullable(r#type) => self.check_type(ctx, r#type.inner),
            Type::Array(r#type) => {
                if self.cfg.check_generic_array && has_single_parameter(r#type.parameters.as_ref()) {
                    self.report_generic_array_type(ctx, r#type.span(), "array");
                }

                self.check_optional_generic_parameters(ctx, r#type.parameters.as_ref());
            }
            Type::NonEmptyArray(r#type) => {
                if self.cfg.check_generic_array && has_single_parameter(r#type.parameters.as_ref()) {
                    self.report_generic_array_type(ctx, r#type.span(), "non-empty-array");
                }

                self.check_optional_generic_parameters(ctx, r#type.parameters.as_ref());
            }
            Type::AssociativeArray(r#type) => {
                if self.cfg.check_generic_array && has_single_parameter(r#type.parameters.as_ref()) {
                    self.report_generic_array_type(ctx, r#type.span(), "associative-array");
                }

                self.check_optional_generic_parameters(ctx, r#type.parameters.as_ref());
            }
            Type::List(r#type) => self.check_optional_generic_parameters(ctx, r#type.parameters.as_ref()),
            Type::NonEmptyList(r#type) => self.check_optional_generic_parameters(ctx, r#type.parameters.as_ref()),
            Type::Iterable(r#type) => {
                if self.cfg.check_generic_array && has_single_parameter(r#type.parameters.as_ref()) {
                    self.report_generic_array_type(ctx, r#type.span(), "iterable");
                }

                self.check_optional_generic_parameters(ctx, r#type.parameters.as_ref());
            }
            Type::ClassString(r#type) => self.check_optional_single_generic_parameter(ctx, r#type.parameter.as_ref()),
            Type::ClassLikeString(r#type) => {
                self.check_optional_single_generic_parameter(ctx, r#type.parameter.as_ref());
            }
            Type::InterfaceString(r#type) => {
                self.check_optional_single_generic_parameter(ctx, r#type.parameter.as_ref())
            }
            Type::EnumString(r#type) => self.check_optional_single_generic_parameter(ctx, r#type.parameter.as_ref()),
            Type::TraitString(r#type) => self.check_optional_single_generic_parameter(ctx, r#type.parameter.as_ref()),
            Type::Reference(r#type) => self.check_optional_generic_parameters(ctx, r#type.parameters.as_ref()),
            Type::Shape(r#type) => {
                for field in r#type.fields.iter() {
                    self.check_type(ctx, field.value);
                }

                if let Some(additional_fields) = &r#type.additional_fields {
                    self.check_optional_generic_parameters(ctx, additional_fields.parameters.as_ref());
                }
            }
            Type::Callable(r#type) => {
                if let Some(specification) = &r#type.specification {
                    for parameter in specification.parameters.entries.iter() {
                        if let Some(parameter_type) = &parameter.parameter_type {
                            self.check_type(ctx, parameter_type);
                        }
                    }

                    if let Some(return_type) = &specification.return_type {
                        self.check_type(ctx, return_type.return_type);
                    }
                }
            }
            Type::Conditional(r#type) => {
                self.check_type(ctx, r#type.subject);
                self.check_type(ctx, r#type.target);
                self.check_type(ctx, r#type.then);
                self.check_type(ctx, r#type.r#else);
            }
            Type::KeyOf(r#type) => self.check_single_generic_parameter(ctx, &r#type.parameter),
            Type::ValueOf(r#type) => self.check_single_generic_parameter(ctx, &r#type.parameter),
            Type::IntMask(r#type) => self.check_generic_parameters(ctx, &r#type.parameters),
            Type::IntMaskOf(r#type) => self.check_single_generic_parameter(ctx, &r#type.parameter),
            Type::New(r#type) => self.check_single_generic_parameter(ctx, &r#type.parameter),
            Type::TemplateType(r#type) => self.check_generic_parameters(ctx, &r#type.parameters),
            Type::IndexAccess(r#type) => {
                self.check_type(ctx, r#type.target);
                self.check_type(ctx, r#type.index);
            }
            Type::Negated(r#type) => self.check_type(ctx, r#type.operand),
            Type::Posited(r#type) => self.check_type(ctx, r#type.operand),
            Type::PropertiesOf(r#type) => self.check_single_generic_parameter(ctx, &r#type.parameter),
            Type::Slice(r#type) => {
                self.report_shorthand_array_type(ctx, r#type.left_bracket.join(r#type.right_bracket));
                self.check_type(ctx, r#type.inner);
            }
            Type::TrailingPipe(r#type) => self.check_type(ctx, r#type.inner),
            Type::Object(r#type) => {
                if let Some(properties) = &r#type.properties {
                    for field in properties.fields.iter() {
                        self.check_type(ctx, field.value);
                    }
                }
            }
            _ => {}
        }
    }

    fn check_optional_generic_parameters<'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        parameters: Option<&GenericParameters<'arena>>,
    ) where
        A: Arena,
    {
        if let Some(parameters) = parameters {
            self.check_generic_parameters(ctx, parameters);
        }
    }

    fn check_generic_parameters<'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        parameters: &GenericParameters<'arena>,
    ) where
        A: Arena,
    {
        for parameter in parameters.entries.iter() {
            self.check_type(ctx, &parameter.inner);
        }
    }

    fn check_optional_single_generic_parameter<'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        parameter: Option<&SingleGenericParameter<'arena>>,
    ) where
        A: Arena,
    {
        if let Some(parameter) = parameter {
            self.check_single_generic_parameter(ctx, parameter);
        }
    }

    fn check_single_generic_parameter<'arena, A>(
        &self,
        ctx: &mut LintContext<'_, 'arena, A>,
        parameter: &SingleGenericParameter<'arena>,
    ) where
        A: Arena,
    {
        self.check_type(ctx, &parameter.entry.inner);
    }

    fn report_shorthand_array_type<A>(&self, ctx: &mut LintContext<'_, '_, A>, span: Span)
    where
        A: Arena,
    {
        let issue = Issue::new(self.cfg.level(), "Legacy shorthand array type syntax is ambiguous.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message("Array shorthand `[]` used here."))
            .with_note(
                "`T[]` means `array<array-key, T>`, but it does not say whether a list or keyed array is intended.",
            )
            .with_help("Use an explicit `array<K, V>` type, or `list<T>` when the value is a sequential list.");

        ctx.collector.report(issue);
    }

    fn report_generic_array_type<A>(&self, ctx: &mut LintContext<'_, '_, A>, span: Span, type_name: &'static str)
    where
        A: Arena,
    {
        let issue = Issue::new(self.cfg.level(), "Generic array type is missing an explicit key type.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(span).with_message(format!("`{type_name}<T>` omits the key type.")))
            .with_note("Single-argument generic array types do not say which key type is expected.")
            .with_help(format!(
                "Use `{type_name}<array-key, T>` or another type that makes the collection shape explicit."
            ));

        ctx.collector.report(issue);
    }
}

fn has_single_parameter(parameters: Option<&GenericParameters<'_>>) -> bool {
    parameters.is_some_and(|parameters| parameters.entries.len() == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;
    use indoc::indoc;

    test_lint_failure! {
        name = flags_param_shorthand_array_type,
        rule = NoShorthandArrayTypeRule,
        count = 1,
        code = indoc! {r"
            <?php

            /** @param User[] $users */
            function hydrate(array $users): void {}
        "}
    }

    test_lint_failure! {
        name = flags_var_and_return_shorthand_array_types,
        rule = NoShorthandArrayTypeRule,
        count = 2,
        code = indoc! {r"
            <?php

            /** @return Bar[] */
            function get_bars(): array
            {
                /** @var Foo[] $foos */
                $foos = [];

                return $foos;
            }
        "}
    }

    test_lint_failure! {
        name = flags_nested_shorthand_array_type,
        rule = NoShorthandArrayTypeRule,
        count = 2,
        code = indoc! {r"
            <?php

            /** @return Foo[][] */
            function get_nested(): array
            {
                return [];
            }
        "}
    }

    test_lint_success! {
        name = allows_explicit_array_and_list_types,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r"
            <?php

            /**
             * @param array<int, User> $users
             * @param list<User> $ordered
             *
             * @return array<array-key, User>
             */
            function get_users(array $users, array $ordered): array
            {
                return $users + $ordered;
            }
        "}
    }

    test_lint_success! {
        name = allows_generic_array_by_default,
        rule = NoShorthandArrayTypeRule,
        code = indoc! {r"
            <?php

            /** @param array<User> $users */
            function hydrate(array $users): void {}
        "}
    }

    test_lint_failure! {
        name = flags_generic_array_when_configured,
        rule = NoShorthandArrayTypeRule,
        count = 2,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_shorthand_array_type.config.check_generic_array = true;
        },
        code = indoc! {r"
            <?php

            /**
             * @param array<User> $users
             *
             * @return iterable<User>
             */
            function iterate(array $users): iterable
            {
                return $users;
            }
        "}
    }

    test_lint_failure! {
        name = flags_array_like_generics_when_configured,
        rule = NoShorthandArrayTypeRule,
        count = 2,
        settings = |s: &mut crate::settings::Settings| {
            s.rules.no_shorthand_array_type.config.check_generic_array = true;
        },
        code = indoc! {r"
            <?php

            /**
             * @param non-empty-array<User> $users
             *
             * @return associative-array<User>
             */
            function hydrate(array $users): array
            {
                return $users;
            }
        "}
    }

    #[test]
    fn rule_is_disabled_by_default() {
        assert!(!<NoShorthandArrayTypeConfig as crate::rule::Config>::default_enabled());
        assert!(!crate::settings::RuleSettings::<NoShorthandArrayTypeConfig>::default_enabled());
    }
}
