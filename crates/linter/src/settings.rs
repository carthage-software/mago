use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use mago_php_version::PHPVersion;

use crate::integration::IntegrationSet;
use crate::rule::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct Settings {
    pub php_version: PHPVersion,
    pub integrations: IntegrationSet,
    pub rules: RulesSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, bound = "C: Serialize + DeserializeOwned")]
pub struct RuleSettings<C: Config> {
    pub enabled: bool,

    #[serde(flatten)]
    pub config: C,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct RulesSettings {
    pub ambiguous_function_call: RuleSettings<AmbiguousFunctionCallConfig>,
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
    pub no_redundant_final: RuleSettings<NoRedundantFinalConfig>,
    pub no_redundant_file: RuleSettings<NoRedundantFileConfig>,
    pub no_redundant_continue: RuleSettings<NoRedundantContinueConfig>,
    pub no_redundant_block: RuleSettings<NoRedundantBlockConfig>,
    pub no_php_tag_terminator: RuleSettings<NoPhpTagTerminatorConfig>,
    pub no_noop: RuleSettings<NoNoopConfig>,
    pub no_multi_assignments: RuleSettings<NoMultiAssignmentsConfig>,
    pub no_nested_ternary: RuleSettings<NoNestedTernaryConfig>,
    pub no_hash_emoji: RuleSettings<NoHashEmojiConfig>,
    pub no_hash_comment: RuleSettings<NoHashCommentConfig>,
    pub no_goto: RuleSettings<NoGotoConfig>,
    pub no_global: RuleSettings<NoGlobalConfig>,
    pub no_ffi: RuleSettings<NoFfiConfig>,
    pub no_eval: RuleSettings<NoEvalConfig>,
    pub no_error_control_operator: RuleSettings<NoErrorControlOperatorConfig>,
    pub no_empty: RuleSettings<NoEmptyConfig>,
    pub no_empty_loop: RuleSettings<NoEmptyLoopConfig>,
    pub no_empty_comment: RuleSettings<NoEmptyCommentConfig>,
    pub no_empty_catch_clause: RuleSettings<NoEmptyCatchClauseConfig>,
    pub no_else_clause: RuleSettings<NoElseClauseConfig>,
    pub no_closing_tag: RuleSettings<NoClosingTagConfig>,
    pub no_boolean_literal_comparison: RuleSettings<NoBooleanLiteralComparisonConfig>,
    pub no_boolean_flag_parameter: RuleSettings<NoBooleanFlagParameterConfig>,
    pub no_assign_in_condition: RuleSettings<NoAssignInConditionConfig>,
    pub no_alias_function: RuleSettings<NoAliasFunctionConfig>,
    pub lowercase_type_hint: RuleSettings<LowercaseTypeHintConfig>,
    pub identity_comparison: RuleSettings<IdentityComparisonConfig>,
    pub interface_name: RuleSettings<InterfaceNameConfig>,
    pub invalid_open_tag: RuleSettings<InvalidOpenTagConfig>,
    pub function_name: RuleSettings<FunctionNameConfig>,
    pub explicit_nullable_param: RuleSettings<ExplicitNullableParamConfig>,
    pub explicit_octal: RuleSettings<ExplicitOctalConfig>,
    pub prefer_arrow_function: RuleSettings<PreferArrowFunctionConfig>,
    pub prefer_interface: RuleSettings<PreferInterfaceConfig>,
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
    pub constant_condition: RuleSettings<ConstantConditionConfig>,
    pub no_ini_set: RuleSettings<NoIniSetConfig>,
    pub no_insecure_comparison: RuleSettings<NoInsecureComparisonConfig>,
    pub no_literal_password: RuleSettings<NoLiteralPasswordConfig>,
    pub tainted_data_to_sink: RuleSettings<TaintedDataToSinkConfig>,
    pub parameter_type: RuleSettings<ParameterTypeConfig>,
    pub property_type: RuleSettings<PropertyTypeConfig>,
    pub no_unsafe_finally: RuleSettings<NoUnsafeFinallyConfig>,
    pub strict_assertions: RuleSettings<StrictAssertionsConfig>,
    pub no_request_all: RuleSettings<NoRequestAllConfig>,
    pub middleware_in_routes: RuleSettings<MiddlewareInRoutesConfig>,
    pub use_compound_assignment: RuleSettings<UseCompoundAssignmentConfig>,
}

impl<C: Config> RuleSettings<C> {
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

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
