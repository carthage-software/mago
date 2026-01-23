use serde::de::DeserializeOwned;

use mago_php_version::PHPVersion;
use mago_reporting::Level;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::context::LintContext;
use crate::integration::IntegrationSet;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use crate::settings::RulesSettings;
use crate::settings::Settings;

pub mod best_practices;
pub mod clarity;
pub mod consistency;
pub mod correctness;
pub mod deprecation;
pub mod maintainability;
pub mod redundancy;
pub mod safety;
pub mod security;

pub use best_practices::*;
pub use clarity::*;
pub use consistency::*;
pub use correctness::*;
pub use deprecation::*;
pub use maintainability::*;
pub use redundancy::*;
pub use safety::*;
pub use security::*;

mod utils;

#[cfg(test)]
mod tests;

pub trait Config: Default + DeserializeOwned {
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

    #[must_use]
    fn deprecated() -> bool {
        false
    }

    #[inline]
    #[must_use]
    fn is_enabled_for(php_version: PHPVersion, integrations: IntegrationSet) -> bool {
        Self::meta().requirements.are_met_by(php_version, integrations)
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self;

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>);
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
            pub fn get_all_for(settings: &Settings, only: Option<&[String]>, include_disabled: bool) -> Vec<Self> {
                let mut rules = Vec::new();

                $(
                    let meta = $rule::meta();

                    // If `--only` is used, check if this rule's code is in the list.
                    if let Some(only_codes) = &only {
                        if only_codes.iter().any(|c| c == meta.code) {
                            rules.push(AnyRule::$variant($rule::build(&settings.rules.$module)));
                        }
                    } else {
                        let is_enabled = include_disabled || (
                            settings.rules.$module.is_enabled()
                            && $rule::is_enabled_for(settings.php_version, settings.integrations)
                        );

                        if is_enabled {
                            rules.push(AnyRule::$variant($rule::build(&settings.rules.$module)));
                        }
                    }
                )*

                rules
            }

            #[inline]
            pub fn name(&self) -> &'static str {
                self.meta().name
            }

            #[inline]
            pub fn code(&self) -> &'static str {
                self.meta().code
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
            pub fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>)  {
                match self {
                    $( AnyRule::$variant(r) => r.check(ctx, node), )*
                }
            }
        }

        /// Filters rule settings based on PHP version and integration requirements.
        ///
        /// Returns a JSON map containing only rules whose requirements are met
        /// by the given PHP version and integrations.
        #[must_use]
        pub fn filter_rules_settings(
            rules: &RulesSettings,
            php_version: PHPVersion,
            integrations: IntegrationSet,
        ) -> serde_json::Map<String, serde_json::Value> {
            let mut map = serde_json::Map::new();
            $(
                if $rule::is_enabled_for(php_version, integrations) {
                    if let Ok(value) = serde_json::to_value(&rules.$module) {
                        map.insert(
                            stringify!($module).replace('_', "-"),
                            value,
                        );
                    }
                }
            )*
            map
        }
    }
}

define_rules! {
    AmbiguousFunctionCall(ambiguous_function_call @ AmbiguousFunctionCallRule),
    UseDedicatedExpectation(use_dedicated_expectation @ UseDedicatedExpectationRule),
    UseSimplerExpectation(use_simpler_expectation @ UseSimplerExpectationRule),
    UseSpecificExpectations(use_specific_expectations @ UseSpecificExpectationsRule),
    NoOnly(no_only @ NoOnlyRule),
    ArrayStyle(array_style @ ArrayStyleRule),
    AssertDescription(assert_description @ AssertDescriptionRule),
    AssertionStyle(assertion_style @ AssertionStyleRule),
    BlockStatement(block_statement @ BlockStatementRule),
    BracedStringInterpolation(braced_string_interpolation @ BracedStringInterpolationRule),
    ClassName(class_name @ ClassNameRule),
    CombineConsecutiveIssets(combine_consecutive_issets @ CombineConsecutiveIssetsRule),
    ConstantName(constant_name @ ConstantNameRule),
    ConstantType(constant_type @ ConstantTypeRule),
    CyclomaticComplexity(cyclomatic_complexity @ CyclomaticComplexityRule),
    DisallowedFunctions(disallowed_functions @ DisallowedFunctionsRule),
    EnumName(enum_name @ EnumNameRule),
    ExcessiveNesting(excessive_nesting @ ExcessiveNestingRule),
    ExcessiveParameterList(excessive_parameter_list @ ExcessiveParameterListRule),
    FinalController(final_controller @ FinalControllerRule),
    Halstead(halstead @ HalsteadRule),
    KanDefect(kan_defect @ KanDefectRule),
    LiteralNamedArgument(literal_named_argument @ LiteralNamedArgumentRule),
    LoopDoesNotIterate(loop_does_not_iterate @ LoopDoesNotIterateRule),
    LowercaseKeyword(lowercase_keyword @ LowercaseKeywordRule),
    NoDebugSymbols(no_debug_symbols @ NoDebugSymbolsRule),
    NoRequestVariable(no_request_variable @ NoRequestVariableRule),
    NoShellExecuteString(no_shell_execute_string @ NoShellExecuteStringRule),
    NoShortOpeningTag(no_short_opening_tag @ NoShortOpeningTagRule),
    NoShorthandTernary(no_shorthand_ternary @ NoShorthandTernaryRule),
    NoSprintfConcat(no_sprintf_concat @ NoSprintfConcatRule),
    OptionalParamOrder(optional_param_order @ OptionalParamOrderRule),
    PreferInterface(prefer_interface @ PreferInterfaceRule),
    PreferAnonymousMigration(prefer_anonymous_migration @ PreferAnonymousMigrationRule),
    PreferFirstClassCallable(prefer_first_class_callable @ PreferFirstClassCallableRule),
    NoVoidReferenceReturn(no_void_reference_return @ NoVoidReferenceReturnRule),
    NoUnderscoreClass(no_underscore_class @ NoUnderscoreClassRule),
    NoTrailingSpace(no_trailing_space @ NoTrailingSpaceRule),
    NoRedundantWriteVisibility(no_redundant_write_visibility @ NoRedundantWriteVisibilityRule),
    NoRedundantStringConcat(no_redundant_string_concat @ NoRedundantStringConcatRule),
    NoRedundantParentheses(no_redundant_parentheses @ NoRedundantParenthesesRule),
    NoRedundantMethodOverride(no_redundant_method_override @ NoRedundantMethodOverrideRule),
    NoRedundantNullsafe(no_redundant_nullsafe @ NoRedundantNullsafeRule),
    NoRedundantMath(no_redundant_math @ NoRedundantMathRule),
    NoRedundantLabel(no_redundant_label @ NoRedundantLabelRule),
    NoRedundantLiteralReturn(no_redundant_literal_return @ NoRedundantLiteralReturnRule),
    NoRedundantFinal(no_redundant_final @ NoRedundantFinalRule),
    NoRedundantReadonly(no_redundant_readonly @ NoRedundantReadonlyRule),
    NoRedundantFile(no_redundant_file @ NoRedundantFileRule),
    NoRedundantContinue(no_redundant_continue @ NoRedundantContinueRule),
    NoRedundantBlock(no_redundant_block @ NoRedundantBlockRule),
    NoRedundantUse(no_redundant_use @ NoRedundantUseRule),
    NoRedundantYieldFrom(no_redundant_yield_from @ NoRedundantYieldFromRule),
    NoSelfAssignment(no_self_assignment @ NoSelfAssignmentRule),
    NoProtectedInFinal(no_protected_in_final @ NoProtectedInFinalRule),
    NoPhpTagTerminator(no_php_tag_terminator @ NoPhpTagTerminatorRule),
    NoNoop(no_noop @ NoNoopRule),
    NoMultiAssignments(no_multi_assignments @ NoMultiAssignmentsRule),
    NoNestedTernary(no_nested_ternary @ NoNestedTernaryRule),
    NoHashEmoji(no_hash_emoji @ NoHashEmojiRule),
    NoHashComment(no_hash_comment @ NoHashCommentRule),
    NoVariableVariable(no_variable_variable @ NoVariableVariableRule),
    NoGoto(no_goto @ NoGotoRule),
    NoGlobal(no_global @ NoGlobalRule),
    NoFfi(no_ffi @ NoFfiRule),
    NoEval(no_eval @ NoEvalRule),
    NoErrorControlOperator(no_error_control_operator @ NoErrorControlOperatorRule),
    NoEmpty(no_empty @ NoEmptyRule),
    NoIsset(no_isset @ NoIssetRule),
    NoEmptyLoop(no_empty_loop @ NoEmptyLoopRule),
    NoEmptyComment(no_empty_comment @ NoEmptyCommentRule),
    NoEmptyCatchClause(no_empty_catch_clause @ NoEmptyCatchClauseRule),
    NoElseClause(no_else_clause @ NoElseClauseRule),
    NoClosingTag(no_closing_tag @ NoClosingTagRule),
    NoBooleanLiteralComparison(no_boolean_literal_comparison @ NoBooleanLiteralComparisonRule),
    NoBooleanFlagParameter(no_boolean_flag_parameter @ NoBooleanFlagParameterRule),
    NoAssignInArgument(no_assign_in_argument @ NoAssignInArgumentRule),
    NoAssignInCondition(no_assign_in_condition @ NoAssignInConditionRule),
    NoAliasFunction(no_alias_function @ NoAliasFunctionRule),
    LowercaseTypeHint(lowercase_type_hint @ LowercaseTypeHintRule),
    IdentityComparison(identity_comparison @ IdentityComparisonRule),
    IneffectiveFormatIgnoreNext(ineffective_format_ignore_next @ IneffectiveFormatIgnoreNextRule),
    InlineVariableReturn(inline_variable_return @ InlineVariableReturnRule),
    IneffectiveFormatIgnoreRegion(ineffective_format_ignore_region @ IneffectiveFormatIgnoreRegionRule),
    InstanceofStringable(instanceof_stringable @ InstanceofStringableRule),
    InterfaceName(interface_name @ InterfaceNameRule),
    InvalidOpenTag(invalid_open_tag @ InvalidOpenTagRule),
    FunctionName(function_name @ FunctionNameRule),
    ExplicitOctal(explicit_octal @ ExplicitOctalRule),
    ReadableLiteral(readable_literal @ ReadableLiteralRule),
    ExplicitNullableParam(explicit_nullable_param @ ExplicitNullableParamRule),
    PreferArrowFunction(prefer_arrow_function @ PreferArrowFunctionRule),
    PreferEarlyContinue(prefer_early_continue @ PreferEarlyContinueRule),
    PreferStaticClosure(prefer_static_closure @ PreferStaticClosureRule),
    PreferViewArray(prefer_view_array @ PreferViewArrayRule),
    PreferWhileLoop(prefer_while_loop @ PreferWhileLoopRule),
    PslArrayFunctions(psl_array_functions @ PslArrayFunctionsRule),
    PslDataStructures(psl_data_structures @ PslDataStructuresRule),
    PslDatetime(psl_datetime @ PslDatetimeRule),
    PslMathFunctions(psl_math_functions @ PslMathFunctionsRule),
    PslOutput(psl_output @ PslOutputRule),
    PslRandomnessFunctions(psl_randomness_functions @ PslRandomnessFunctionsRule),
    PslRegexFunctions(psl_regex_functions @ PslRegexFunctionsRule),
    PslSleepFunctions(psl_sleep_functions @ PslSleepFunctionsRule),
    PslStringFunctions(psl_string_functions @ PslStringFunctionsRule),
    ReturnType(return_type @ ReturnTypeRule),
    StrContains(str_contains @ StrContainsRule),
    StrStartsWith(str_starts_with @ StrStartsWithRule),
    StrictBehavior(strict_behavior @ StrictBehaviorRule),
    StrictTypes(strict_types @ StrictTypesRule),
    TaggedFixme(tagged_fixme @ TaggedFixmeRule),
    TaggedTodo(tagged_todo @ TaggedTodoRule),
    TooManyEnumCases(too_many_enum_cases @ TooManyEnumCasesRule),
    TooManyMethods(too_many_methods @ TooManyMethodsRule),
    TooManyProperties(too_many_properties @ TooManyPropertiesRule),
    TraitName(trait_name @ TraitNameRule),
    ValidDocblock(valid_docblock @ ValidDocblockRule),
    VariableName(variable_name @ VariableNameRule),
    ConstantCondition(constant_condition @ ConstantConditionRule),
    NoIniSet(no_ini_set @ NoIniSetRule),
    NoInsecureComparison(no_insecure_comparison @ NoInsecureComparisonRule),
    NoLiteralPassword(no_literal_password @ NoLiteralPasswordRule),
    TaintedDataToSink(tainted_data_to_sink @ TaintedDataToSinkRule),
    SensitiveParameter(sensitive_parameter @ SensitiveParameterRule),
    ParameterType(parameter_type @ ParameterTypeRule),
    PropertyName(property_name @ PropertyNameRule),
    PropertyType(property_type @ PropertyTypeRule),
    NoUnsafeFinally(no_unsafe_finally @ NoUnsafeFinallyRule),
    StrictAssertions(strict_assertions @ StrictAssertionsRule),
    UseSpecificAssertions(use_specific_assertions @ UseSpecificAssertionsRule),
    NoRequestAll(no_request_all @ NoRequestAllRule),
    MiddlewareInRoutes(middleware_in_routes @ MiddlewareInRoutesRule),
    UseCompoundAssignment(use_compound_assignment @ UseCompoundAssignmentRule),
    RequirePregQuoteDelimiter(require_preg_quote_delimiter @ RequirePregQuoteDelimiterRule),
    RequireNamespace(require_namespace @ RequireNamespaceRule),
    YodaConditions(yoda_conditions @ YodaConditionsRule),
    UseWpFunctions(use_wp_functions @ UseWpFunctionsRule),
    NoDirectDbQuery(no_direct_db_query @ NoDirectDbQueryRule),
    NoDbSchemaChange(no_db_schema_change @ NoDbSchemaChangeRule),
    NoUnescapedOutput(no_unescaped_output @ NoUnescapedOutputRule),
    NoRolesAsCapabilities(no_roles_as_capabilities @ NoRolesAsCapabilitiesRule),
}
