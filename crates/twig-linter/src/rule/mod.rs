use mago_allocator::Arena;

use mago_reporting::Level;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::context::LintContext;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use crate::settings::Settings;
use crate::version::TwigVersion;

pub mod best_practices;
pub mod clarity;
pub mod consistency;
pub mod correctness;
pub mod deprecation;
pub mod maintainability;
pub mod redundancy;
pub mod security;

pub use best_practices::*;
pub use clarity::*;
pub use consistency::*;
pub use correctness::*;
pub use deprecation::*;
pub use maintainability::*;
pub use redundancy::*;
pub use security::*;

mod utils;

#[cfg(test)]
mod tests;

#[cfg(feature = "serde")]
pub trait Config: Default + serde::de::DeserializeOwned {
    /// Whether the rule is enabled by default.
    #[must_use]
    fn default_enabled() -> bool {
        true
    }

    /// The severity level of the rule.
    fn level(&self) -> Level;
}

#[cfg(not(feature = "serde"))]
pub trait Config: Default {
    /// Whether the rule is enabled by default.
    #[must_use]
    fn default_enabled() -> bool {
        true
    }

    /// The severity level of the rule.
    fn level(&self) -> Level;
}

pub trait LintRule {
    type Config: Config;

    fn meta() -> &'static RuleMeta;

    fn targets() -> &'static [NodeKind];

    #[inline]
    #[must_use]
    fn is_enabled_for(version: TwigVersion) -> bool {
        Self::meta().requirements.are_met_by(version)
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self;

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena;
}

macro_rules! define_rules {
    ($(

        $variant:ident(
            $module:ident @ $rule:ident
        )

    ),* $(,)?) => {
        #[derive(Debug, Clone)]
        pub enum AnyRule {$(
            $variant($rule),
        )*}

        impl AnyRule {
            /// Builds the rules selected by the settings.
            ///
            /// A rule that needs a newer Twig version than the configured minimum is never
            /// selected: its fixes could produce syntax that older Twig versions reject.
            pub fn get_all_for(settings: &Settings, only: Option<&[String]>, include_disabled: bool) -> Vec<(Self, Vec<String>)> {
                let mut rules = Vec::new();

                $(
                    let meta = $rule::meta();

                    if $rule::is_enabled_for(settings.version) {
                        let is_selected = match &only {
                            Some(only_codes) => only_codes.iter().any(|c| c == meta.code),
                            None => include_disabled || settings.rules.$module.is_enabled(),
                        };

                        if is_selected {
                            rules.push((AnyRule::$variant($rule::build(&settings.rules.$module)), settings.rules.$module.exclude.clone()));
                        }
                    }
                )*

                rules
            }

            #[inline]
            pub fn code(&self) -> &'static str {
                self.meta().code
            }

            /// The level this rule reports at, as configured.
            #[inline]
            pub fn level(&self) -> Level {
                match self {
                    $( AnyRule::$variant(r) => r.cfg.level(), )*
                }
            }

            #[inline]
            pub fn default_level(&self) -> Level {
                match self {
                    $( AnyRule::$variant(_) => <$rule as LintRule>::Config::default().level(), )*
                }
            }

            #[inline]
            pub fn meta(&self) -> &'static RuleMeta {
                match self {
                    $( AnyRule::$variant(_) => $rule::meta(), )*
                }
            }

            #[inline]
            pub fn targets(&self) -> &'static [NodeKind] {
                match self {
                    $( AnyRule::$variant(_) => $rule::targets(), )*
                }
            }

            #[inline]
            pub fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
            where
                A: Arena,
            {
                match self {
                    $( AnyRule::$variant(r) => r.check(ctx, node), )*
                }
            }
        }
    }
}

define_rules! {
    DuplicateBlock(duplicate_block @ DuplicateBlockRule),
    DuplicateParameter(duplicate_parameter @ DuplicateParameterRule),
    DuplicateArgument(duplicate_argument @ DuplicateArgumentRule),
    DuplicateKey(duplicate_key @ DuplicateKeyRule),
    DuplicateMacro(duplicate_macro @ DuplicateMacroRule),
    DeprecatedSpaceless(deprecated_spaceless @ DeprecatedSpacelessRule),
    DeprecatedAttribute(deprecated_attribute @ DeprecatedAttributeRule),
    NamedArgumentSeparator(named_argument_separator @ NamedArgumentSeparatorRule),
    VariableName(variable_name @ VariableNameRule),
    MacroName(macro_name @ MacroNameRule),
    MacroArgumentName(macro_argument_name @ MacroArgumentNameRule),
    NamedArgumentName(named_argument_name @ NamedArgumentNameRule),
    EndBlockName(end_block_name @ EndBlockNameRule),
    SingleQuotes(single_quotes @ SingleQuotesRule),
    NoDump(no_dump @ NoDumpRule),
    NoRaw(no_raw @ NoRawRule),
    NoAutoescapeFalse(no_autoescape_false @ NoAutoescapeFalseRule),
    DisallowedFunctions(disallowed_functions @ DisallowedFunctionsRule),
    DisallowedFilters(disallowed_filters @ DisallowedFiltersRule),
    DisallowedTags(disallowed_tags @ DisallowedTagsRule),
    ExplicitIncludeContext(explicit_include_context @ ExplicitIncludeContextRule),
    NoIncludeIgnoreMissing(no_include_ignore_missing @ NoIncludeIgnoreMissingRule),
    PreferIncludeFunction(prefer_include_function @ PreferIncludeFunctionRule),
    NoNestedTernary(no_nested_ternary @ NoNestedTernaryRule),
    ConstantCondition(constant_condition @ ConstantConditionRule),
}
