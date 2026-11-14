use schemars::JsonSchema;

use mago_database::GlobSettings;

use crate::rule::Config;
use crate::rule::ConstantConditionConfig;
use crate::rule::DeprecatedAttributeConfig;
use crate::rule::DeprecatedSpacelessConfig;
use crate::rule::DisallowedFiltersConfig;
use crate::rule::DisallowedFunctionsConfig;
use crate::rule::DisallowedTagsConfig;
use crate::rule::DuplicateArgumentConfig;
use crate::rule::DuplicateBlockConfig;
use crate::rule::DuplicateKeyConfig;
use crate::rule::DuplicateMacroConfig;
use crate::rule::DuplicateParameterConfig;
use crate::rule::EndBlockNameConfig;
use crate::rule::ExplicitIncludeContextConfig;
use crate::rule::MacroArgumentNameConfig;
use crate::rule::MacroNameConfig;
use crate::rule::NamedArgumentNameConfig;
use crate::rule::NamedArgumentSeparatorConfig;
use crate::rule::NoAutoescapeFalseConfig;
use crate::rule::NoDumpConfig;
use crate::rule::NoIncludeIgnoreMissingConfig;
use crate::rule::NoNestedTernaryConfig;
use crate::rule::NoRawConfig;
use crate::rule::PreferIncludeFunctionConfig;
use crate::rule::SingleQuotesConfig;
use crate::rule::VariableNameConfig;
use crate::version::TwigVersion;

#[derive(Debug, Clone, Default, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct Settings {
    /// The oldest Twig 3 version the project supports. Rules that need a newer version stay off.
    pub version: TwigVersion,
    pub rules: RulesSettings,
    #[schemars(skip)]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub glob: GlobSettings,
}

#[derive(Debug, Clone, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, deny_unknown_fields, bound = "C: serde::Serialize + serde::de::DeserializeOwned")
)]
#[schemars(bound = "C: JsonSchema")]
pub struct RuleSettings<C: Config> {
    pub enabled: bool,

    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub exclude: Vec<String>,

    #[cfg_attr(feature = "serde", serde(flatten))]
    pub config: C,
}

#[derive(Debug, Clone, Default, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct RulesSettings {
    pub duplicate_block: RuleSettings<DuplicateBlockConfig>,
    pub duplicate_parameter: RuleSettings<DuplicateParameterConfig>,
    pub duplicate_argument: RuleSettings<DuplicateArgumentConfig>,
    pub duplicate_key: RuleSettings<DuplicateKeyConfig>,
    pub duplicate_macro: RuleSettings<DuplicateMacroConfig>,
    pub deprecated_spaceless: RuleSettings<DeprecatedSpacelessConfig>,
    pub deprecated_attribute: RuleSettings<DeprecatedAttributeConfig>,
    pub named_argument_separator: RuleSettings<NamedArgumentSeparatorConfig>,
    pub variable_name: RuleSettings<VariableNameConfig>,
    pub macro_name: RuleSettings<MacroNameConfig>,
    pub macro_argument_name: RuleSettings<MacroArgumentNameConfig>,
    pub named_argument_name: RuleSettings<NamedArgumentNameConfig>,
    pub end_block_name: RuleSettings<EndBlockNameConfig>,
    pub single_quotes: RuleSettings<SingleQuotesConfig>,
    pub no_dump: RuleSettings<NoDumpConfig>,
    pub no_raw: RuleSettings<NoRawConfig>,
    pub no_autoescape_false: RuleSettings<NoAutoescapeFalseConfig>,
    pub disallowed_functions: RuleSettings<DisallowedFunctionsConfig>,
    pub disallowed_filters: RuleSettings<DisallowedFiltersConfig>,
    pub disallowed_tags: RuleSettings<DisallowedTagsConfig>,
    pub explicit_include_context: RuleSettings<ExplicitIncludeContextConfig>,
    pub no_include_ignore_missing: RuleSettings<NoIncludeIgnoreMissingConfig>,
    pub prefer_include_function: RuleSettings<PreferIncludeFunctionConfig>,
    pub no_nested_ternary: RuleSettings<NoNestedTernaryConfig>,
    pub constant_condition: RuleSettings<ConstantConditionConfig>,
}

impl<C: Config> RuleSettings<C> {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl<C: Config> Default for RuleSettings<C> {
    fn default() -> Self {
        Self { enabled: C::default_enabled(), exclude: Vec::new(), config: C::default() }
    }
}
