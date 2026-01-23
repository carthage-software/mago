use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use mago_php_version::PHPVersion;

use crate::integration::IntegrationSet;
use crate::rule::AmbiguousFunctionCallConfig;
use crate::rule::ArrayStyleConfig;
use crate::rule::AssertDescriptionConfig;
use crate::rule::AssertionStyleConfig;
use crate::rule::BlockStatementConfig;
use crate::rule::BracedStringInterpolationConfig;
use crate::rule::ClassNameConfig;
use crate::rule::CombineConsecutiveIssetsConfig;
use crate::rule::Config;
use crate::rule::ConstantConditionConfig;
use crate::rule::ConstantNameConfig;
use crate::rule::ConstantTypeConfig;
use crate::rule::CyclomaticComplexityConfig;
use crate::rule::DisallowedFunctionsConfig;
use crate::rule::EnumNameConfig;
use crate::rule::ExcessiveNestingConfig;
use crate::rule::ExcessiveParameterListConfig;
use crate::rule::ExplicitNullableParamConfig;
use crate::rule::ExplicitOctalConfig;
use crate::rule::FinalControllerConfig;
use crate::rule::FunctionNameConfig;
use crate::rule::HalsteadConfig;
use crate::rule::IdentityComparisonConfig;
use crate::rule::IneffectiveFormatIgnoreNextConfig;
use crate::rule::IneffectiveFormatIgnoreRegionConfig;
use crate::rule::InlineVariableReturnConfig;
use crate::rule::InstanceofStringableConfig;
use crate::rule::InterfaceNameConfig;
use crate::rule::InvalidOpenTagConfig;
use crate::rule::KanDefectConfig;
use crate::rule::LiteralNamedArgumentConfig;
use crate::rule::LoopDoesNotIterateConfig;
use crate::rule::LowercaseKeywordConfig;
use crate::rule::LowercaseTypeHintConfig;
use crate::rule::MiddlewareInRoutesConfig;
use crate::rule::NoAliasFunctionConfig;
use crate::rule::NoAssignInArgumentConfig;
use crate::rule::NoAssignInConditionConfig;
use crate::rule::NoBooleanFlagParameterConfig;
use crate::rule::NoBooleanLiteralComparisonConfig;
use crate::rule::NoClosingTagConfig;
use crate::rule::NoDbSchemaChangeConfig;
use crate::rule::NoDebugSymbolsConfig;
use crate::rule::NoDirectDbQueryConfig;
use crate::rule::NoElseClauseConfig;
use crate::rule::NoEmptyCatchClauseConfig;
use crate::rule::NoEmptyCommentConfig;
use crate::rule::NoEmptyConfig;
use crate::rule::NoEmptyLoopConfig;
use crate::rule::NoErrorControlOperatorConfig;
use crate::rule::NoEvalConfig;
use crate::rule::NoFfiConfig;
use crate::rule::NoGlobalConfig;
use crate::rule::NoGotoConfig;
use crate::rule::NoHashCommentConfig;
use crate::rule::NoHashEmojiConfig;
use crate::rule::NoIniSetConfig;
use crate::rule::NoInsecureComparisonConfig;
use crate::rule::NoIssetConfig;
use crate::rule::NoLiteralPasswordConfig;
use crate::rule::NoMultiAssignmentsConfig;
use crate::rule::NoNestedTernaryConfig;
use crate::rule::NoNoopConfig;
use crate::rule::NoOnlyConfig;
use crate::rule::NoPhpTagTerminatorConfig;
use crate::rule::NoProtectedInFinalConfig;
use crate::rule::NoRedundantBlockConfig;
use crate::rule::NoRedundantContinueConfig;
use crate::rule::NoRedundantFileConfig;
use crate::rule::NoRedundantFinalConfig;
use crate::rule::NoRedundantLabelConfig;
use crate::rule::NoRedundantLiteralReturnConfig;
use crate::rule::NoRedundantMathConfig;
use crate::rule::NoRedundantMethodOverrideConfig;
use crate::rule::NoRedundantNullsafeConfig;
use crate::rule::NoRedundantParenthesesConfig;
use crate::rule::NoRedundantReadonlyConfig;
use crate::rule::NoRedundantStringConcatConfig;
use crate::rule::NoRedundantUseConfig;
use crate::rule::NoRedundantWriteVisibilityConfig;
use crate::rule::NoRedundantYieldFromConfig;
use crate::rule::NoRequestAllConfig;
use crate::rule::NoRequestVariableConfig;
use crate::rule::NoRolesAsCapabilitiesConfig;
use crate::rule::NoSelfAssignmentConfig;
use crate::rule::NoShellExecuteStringConfig;
use crate::rule::NoShortOpeningTagConfig;
use crate::rule::NoShorthandTernaryConfig;
use crate::rule::NoSprintfConcatConfig;
use crate::rule::NoTrailingSpaceConfig;
use crate::rule::NoUnderscoreClassConfig;
use crate::rule::NoUnescapedOutputConfig;
use crate::rule::NoUnsafeFinallyConfig;
use crate::rule::NoVariableVariableConfig;
use crate::rule::NoVoidReferenceReturnConfig;
use crate::rule::OptionalParamOrderConfig;
use crate::rule::ParameterTypeConfig;
use crate::rule::PreferAnonymousMigrationConfig;
use crate::rule::PreferArrowFunctionConfig;
use crate::rule::PreferEarlyContinueConfig;
use crate::rule::PreferFirstClassCallableConfig;
use crate::rule::PreferInterfaceConfig;
use crate::rule::PreferStaticClosureConfig;
use crate::rule::PreferViewArrayConfig;
use crate::rule::PreferWhileLoopConfig;
use crate::rule::PropertyNameConfig;
use crate::rule::PropertyTypeConfig;
use crate::rule::PslArrayFunctionsConfig;
use crate::rule::PslDataStructuresConfig;
use crate::rule::PslDatetimeConfig;
use crate::rule::PslMathFunctionsConfig;
use crate::rule::PslOutputConfig;
use crate::rule::PslRandomnessFunctionsConfig;
use crate::rule::PslRegexFunctionsConfig;
use crate::rule::PslSleepFunctionsConfig;
use crate::rule::PslStringFunctionsConfig;
use crate::rule::ReadableLiteralConfig;
use crate::rule::RequireNamespaceConfig;
use crate::rule::RequirePregQuoteDelimiterConfig;
use crate::rule::ReturnTypeConfig;
use crate::rule::SensitiveParameterConfig;
use crate::rule::StrContainsConfig;
use crate::rule::StrStartsWithConfig;
use crate::rule::StrictAssertionsConfig;
use crate::rule::StrictBehaviorConfig;
use crate::rule::StrictTypesConfig;
use crate::rule::TaggedFixmeConfig;
use crate::rule::TaggedTodoConfig;
use crate::rule::TaintedDataToSinkConfig;
use crate::rule::TooManyEnumCasesConfig;
use crate::rule::TooManyMethodsConfig;
use crate::rule::TooManyPropertiesConfig;
use crate::rule::TraitNameConfig;
use crate::rule::UseCompoundAssignmentConfig;
use crate::rule::UseDedicatedExpectationConfig;
use crate::rule::UseSimplerExpectationConfig;
use crate::rule::UseSpecificAssertionsConfig;
use crate::rule::UseSpecificExpectationsConfig;
use crate::rule::UseWpFunctionsConfig;
use crate::rule::ValidDocblockConfig;
use crate::rule::VariableNameConfig;
use crate::rule::YodaConditionsConfig;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct Settings {
    pub php_version: PHPVersion,
    pub integrations: IntegrationSet,
    pub rules: RulesSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields, bound = "C: Serialize + DeserializeOwned")]
#[schemars(bound = "C: JsonSchema")]
pub struct RuleSettings<C: Config> {
    pub enabled: bool,

    #[serde(flatten)]
    pub config: C,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RulesSettings {
    pub ambiguous_function_call: RuleSettings<AmbiguousFunctionCallConfig>,
    pub use_dedicated_expectation: RuleSettings<UseDedicatedExpectationConfig>,
    pub use_simpler_expectation: RuleSettings<UseSimplerExpectationConfig>,
    pub use_specific_expectations: RuleSettings<UseSpecificExpectationsConfig>,
    pub array_style: RuleSettings<ArrayStyleConfig>,
    pub assert_description: RuleSettings<AssertDescriptionConfig>,
    pub assertion_style: RuleSettings<AssertionStyleConfig>,
    pub block_statement: RuleSettings<BlockStatementConfig>,
    pub braced_string_interpolation: RuleSettings<BracedStringInterpolationConfig>,
    pub class_name: RuleSettings<ClassNameConfig>,
    pub combine_consecutive_issets: RuleSettings<CombineConsecutiveIssetsConfig>,
    pub constant_name: RuleSettings<ConstantNameConfig>,
    pub constant_type: RuleSettings<ConstantTypeConfig>,
    pub cyclomatic_complexity: RuleSettings<CyclomaticComplexityConfig>,
    pub disallowed_functions: RuleSettings<DisallowedFunctionsConfig>,
    pub enum_name: RuleSettings<EnumNameConfig>,
    pub excessive_nesting: RuleSettings<ExcessiveNestingConfig>,
    pub excessive_parameter_list: RuleSettings<ExcessiveParameterListConfig>,
    pub final_controller: RuleSettings<FinalControllerConfig>,
    pub halstead: RuleSettings<HalsteadConfig>,
    pub kan_defect: RuleSettings<KanDefectConfig>,
    pub literal_named_argument: RuleSettings<LiteralNamedArgumentConfig>,
    pub loop_does_not_iterate: RuleSettings<LoopDoesNotIterateConfig>,
    pub lowercase_keyword: RuleSettings<LowercaseKeywordConfig>,
    pub no_debug_symbols: RuleSettings<NoDebugSymbolsConfig>,
    pub no_request_variable: RuleSettings<NoRequestVariableConfig>,
    pub no_shell_execute_string: RuleSettings<NoShellExecuteStringConfig>,
    pub no_short_opening_tag: RuleSettings<NoShortOpeningTagConfig>,
    pub no_shorthand_ternary: RuleSettings<NoShorthandTernaryConfig>,
    pub no_sprintf_concat: RuleSettings<NoSprintfConcatConfig>,
    pub optional_param_order: RuleSettings<OptionalParamOrderConfig>,
    pub prefer_anonymous_migration: RuleSettings<PreferAnonymousMigrationConfig>,
    pub prefer_first_class_callable: RuleSettings<PreferFirstClassCallableConfig>,
    pub no_void_reference_return: RuleSettings<NoVoidReferenceReturnConfig>,
    pub no_underscore_class: RuleSettings<NoUnderscoreClassConfig>,
    pub no_trailing_space: RuleSettings<NoTrailingSpaceConfig>,
    pub no_redundant_write_visibility: RuleSettings<NoRedundantWriteVisibilityConfig>,
    pub no_redundant_string_concat: RuleSettings<NoRedundantStringConcatConfig>,
    pub no_redundant_parentheses: RuleSettings<NoRedundantParenthesesConfig>,
    pub no_redundant_method_override: RuleSettings<NoRedundantMethodOverrideConfig>,
    pub no_redundant_nullsafe: RuleSettings<NoRedundantNullsafeConfig>,
    pub no_redundant_math: RuleSettings<NoRedundantMathConfig>,
    pub no_redundant_label: RuleSettings<NoRedundantLabelConfig>,
    pub no_redundant_literal_return: RuleSettings<NoRedundantLiteralReturnConfig>,
    pub no_redundant_final: RuleSettings<NoRedundantFinalConfig>,
    pub no_redundant_readonly: RuleSettings<NoRedundantReadonlyConfig>,
    pub no_redundant_file: RuleSettings<NoRedundantFileConfig>,
    pub no_redundant_continue: RuleSettings<NoRedundantContinueConfig>,
    pub no_redundant_block: RuleSettings<NoRedundantBlockConfig>,
    pub no_redundant_use: RuleSettings<NoRedundantUseConfig>,
    pub no_redundant_yield_from: RuleSettings<NoRedundantYieldFromConfig>,
    pub no_self_assignment: RuleSettings<NoSelfAssignmentConfig>,
    pub no_protected_in_final: RuleSettings<NoProtectedInFinalConfig>,
    pub no_php_tag_terminator: RuleSettings<NoPhpTagTerminatorConfig>,
    pub no_noop: RuleSettings<NoNoopConfig>,
    pub no_only: RuleSettings<NoOnlyConfig>,
    pub no_multi_assignments: RuleSettings<NoMultiAssignmentsConfig>,
    pub no_nested_ternary: RuleSettings<NoNestedTernaryConfig>,
    pub no_hash_emoji: RuleSettings<NoHashEmojiConfig>,
    pub no_hash_comment: RuleSettings<NoHashCommentConfig>,
    pub no_variable_variable: RuleSettings<NoVariableVariableConfig>,
    pub no_goto: RuleSettings<NoGotoConfig>,
    pub no_global: RuleSettings<NoGlobalConfig>,
    pub no_ffi: RuleSettings<NoFfiConfig>,
    pub no_eval: RuleSettings<NoEvalConfig>,
    pub no_error_control_operator: RuleSettings<NoErrorControlOperatorConfig>,
    pub no_empty: RuleSettings<NoEmptyConfig>,
    pub no_isset: RuleSettings<NoIssetConfig>,
    pub no_empty_loop: RuleSettings<NoEmptyLoopConfig>,
    pub no_empty_comment: RuleSettings<NoEmptyCommentConfig>,
    pub no_empty_catch_clause: RuleSettings<NoEmptyCatchClauseConfig>,
    pub no_else_clause: RuleSettings<NoElseClauseConfig>,
    pub no_closing_tag: RuleSettings<NoClosingTagConfig>,
    pub no_boolean_literal_comparison: RuleSettings<NoBooleanLiteralComparisonConfig>,
    pub no_boolean_flag_parameter: RuleSettings<NoBooleanFlagParameterConfig>,
    pub no_assign_in_argument: RuleSettings<NoAssignInArgumentConfig>,
    pub no_assign_in_condition: RuleSettings<NoAssignInConditionConfig>,
    pub no_alias_function: RuleSettings<NoAliasFunctionConfig>,
    pub lowercase_type_hint: RuleSettings<LowercaseTypeHintConfig>,
    pub identity_comparison: RuleSettings<IdentityComparisonConfig>,
    pub ineffective_format_ignore_next: RuleSettings<IneffectiveFormatIgnoreNextConfig>,
    pub ineffective_format_ignore_region: RuleSettings<IneffectiveFormatIgnoreRegionConfig>,
    pub inline_variable_return: RuleSettings<InlineVariableReturnConfig>,
    pub instanceof_stringable: RuleSettings<InstanceofStringableConfig>,
    pub interface_name: RuleSettings<InterfaceNameConfig>,
    pub invalid_open_tag: RuleSettings<InvalidOpenTagConfig>,
    pub function_name: RuleSettings<FunctionNameConfig>,
    pub explicit_nullable_param: RuleSettings<ExplicitNullableParamConfig>,
    pub explicit_octal: RuleSettings<ExplicitOctalConfig>,
    pub prefer_arrow_function: RuleSettings<PreferArrowFunctionConfig>,
    pub prefer_early_continue: RuleSettings<PreferEarlyContinueConfig>,
    pub prefer_interface: RuleSettings<PreferInterfaceConfig>,
    pub prefer_static_closure: RuleSettings<PreferStaticClosureConfig>,
    pub prefer_view_array: RuleSettings<PreferViewArrayConfig>,
    pub prefer_while_loop: RuleSettings<PreferWhileLoopConfig>,
    pub psl_array_functions: RuleSettings<PslArrayFunctionsConfig>,
    pub psl_data_structures: RuleSettings<PslDataStructuresConfig>,
    pub psl_datetime: RuleSettings<PslDatetimeConfig>,
    pub psl_math_functions: RuleSettings<PslMathFunctionsConfig>,
    pub psl_output: RuleSettings<PslOutputConfig>,
    pub psl_randomness_functions: RuleSettings<PslRandomnessFunctionsConfig>,
    pub psl_regex_functions: RuleSettings<PslRegexFunctionsConfig>,
    pub psl_sleep_functions: RuleSettings<PslSleepFunctionsConfig>,
    pub psl_string_functions: RuleSettings<PslStringFunctionsConfig>,
    pub return_type: RuleSettings<ReturnTypeConfig>,
    pub str_contains: RuleSettings<StrContainsConfig>,
    pub str_starts_with: RuleSettings<StrStartsWithConfig>,
    pub strict_behavior: RuleSettings<StrictBehaviorConfig>,
    pub strict_types: RuleSettings<StrictTypesConfig>,
    pub tagged_fixme: RuleSettings<TaggedFixmeConfig>,
    pub tagged_todo: RuleSettings<TaggedTodoConfig>,
    pub too_many_enum_cases: RuleSettings<TooManyEnumCasesConfig>,
    pub too_many_methods: RuleSettings<TooManyMethodsConfig>,
    pub too_many_properties: RuleSettings<TooManyPropertiesConfig>,
    pub trait_name: RuleSettings<TraitNameConfig>,
    pub valid_docblock: RuleSettings<ValidDocblockConfig>,
    pub variable_name: RuleSettings<VariableNameConfig>,
    pub constant_condition: RuleSettings<ConstantConditionConfig>,
    pub no_ini_set: RuleSettings<NoIniSetConfig>,
    pub no_insecure_comparison: RuleSettings<NoInsecureComparisonConfig>,
    pub no_literal_password: RuleSettings<NoLiteralPasswordConfig>,
    pub tainted_data_to_sink: RuleSettings<TaintedDataToSinkConfig>,
    pub sensitive_parameter: RuleSettings<SensitiveParameterConfig>,
    pub parameter_type: RuleSettings<ParameterTypeConfig>,
    pub property_name: RuleSettings<PropertyNameConfig>,
    pub property_type: RuleSettings<PropertyTypeConfig>,
    pub no_unsafe_finally: RuleSettings<NoUnsafeFinallyConfig>,
    pub strict_assertions: RuleSettings<StrictAssertionsConfig>,
    pub use_specific_assertions: RuleSettings<UseSpecificAssertionsConfig>,
    pub no_request_all: RuleSettings<NoRequestAllConfig>,
    pub middleware_in_routes: RuleSettings<MiddlewareInRoutesConfig>,
    pub use_compound_assignment: RuleSettings<UseCompoundAssignmentConfig>,
    pub require_preg_quote_delimiter: RuleSettings<RequirePregQuoteDelimiterConfig>,
    pub require_namespace: RuleSettings<RequireNamespaceConfig>,
    pub readable_literal: RuleSettings<ReadableLiteralConfig>,
    pub yoda_conditions: RuleSettings<YodaConditionsConfig>,
    pub use_wp_functions: RuleSettings<UseWpFunctionsConfig>,
    pub no_direct_db_query: RuleSettings<NoDirectDbQueryConfig>,
    pub no_db_schema_change: RuleSettings<NoDbSchemaChangeConfig>,
    pub no_unescaped_output: RuleSettings<NoUnescapedOutputConfig>,
    pub no_roles_as_capabilities: RuleSettings<NoRolesAsCapabilitiesConfig>,
}

impl<C: Config> RuleSettings<C> {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub fn default_enabled() -> bool {
        C::default_enabled()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self { php_version: PHPVersion::PHP80, integrations: IntegrationSet::empty(), rules: RulesSettings::default() }
    }
}

impl<C: Config> Default for RuleSettings<C> {
    fn default() -> Self {
        Self { enabled: C::default_enabled(), config: C::default() }
    }
}
