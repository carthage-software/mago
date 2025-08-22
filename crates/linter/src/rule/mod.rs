use serde::de::DeserializeOwned;

use mago_php_version::PHPVersion;
use mago_reporting::Level;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::context::LintContext;
use crate::integration::IntegrationSet;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;
use crate::settings::Settings;

pub mod ambiguous_function_call;
pub mod array_style;
pub mod assert_description;
pub mod assertion_style;
pub mod block_statement;
pub mod braced_string_interpolation;
pub mod class_name;
pub mod combine_consecutive_issets;
pub mod constant_condition;
pub mod constant_name;
pub mod constant_type;
pub mod cyclomatic_complexity;
pub mod disallowed_functions;
pub mod enum_name;
pub mod excessive_nesting;
pub mod excessive_parameter_list;
pub mod explicit_nullable_param;
pub mod explicit_octal;
pub mod function_name;
pub mod halstead;
pub mod identity_comparison;
pub mod interface_name;
pub mod kan_defect;
pub mod literal_named_argument;
pub mod loop_does_not_iterate;
pub mod lowercase_keyword;
pub mod lowercase_type_hint;
pub mod middleware_in_routes;
pub mod no_alias_function;
pub mod no_assign_in_condition;
pub mod no_boolean_flag_parameter;
pub mod no_boolean_literal_comparison;
pub mod no_closing_tag;
pub mod no_debug_symbols;
pub mod no_else_clause;
pub mod no_empty;
pub mod no_empty_catch_clause;
pub mod no_empty_comment;
pub mod no_empty_loop;
pub mod no_error_control_operator;
pub mod no_eval;
pub mod no_ffi;
pub mod no_global;
pub mod no_goto;
pub mod no_hash_comment;
pub mod no_hash_emoji;
pub mod no_insecure_comparison;
pub mod no_literal_password;
pub mod no_multi_assignments;
pub mod no_noop;
pub mod no_php_tag_terminator;
pub mod no_redundant_block;
pub mod no_redundant_continue;
pub mod no_redundant_file;
pub mod no_redundant_final;
pub mod no_redundant_label;
pub mod no_redundant_math;
pub mod no_redundant_method_override;
pub mod no_redundant_parentheses;
pub mod no_redundant_string_concat;
pub mod no_redundant_write_visibility;
pub mod no_request_all;
pub mod no_request_variable;
pub mod no_shell_execute_string;
pub mod no_short_opening_tag;
pub mod no_shorthand_ternary;
pub mod no_sprintf_concat;
pub mod no_trailing_space;
pub mod no_underscore_class;
pub mod no_unsafe_finally;
pub mod no_void_reference_return;
pub mod optional_param_order;
pub mod parameter_type;
pub mod prefer_anonymous_migration;
pub mod prefer_interface;
pub mod prefer_view_array;
pub mod prefer_while_loop;
pub mod property_type;
pub mod psl_array_functions;
pub mod psl_data_structures;
pub mod psl_datetime;
pub mod psl_math_functions;
pub mod psl_output;
pub mod psl_randomness_functions;
pub mod psl_regex_functions;
pub mod psl_sleep_functions;
pub mod psl_string_functions;
pub mod return_type;
pub mod str_contains;
pub mod str_starts_with;
pub mod strict_assertions;
pub mod strict_behavior;
pub mod strict_types;
pub mod tagged_fixme;
pub mod tagged_todo;
pub mod tainted_data_to_sink;
pub mod too_many_enum_cases;
pub mod too_many_methods;
pub mod too_many_properties;
pub mod trait_name;
pub mod valid_docblock;

pub use ambiguous_function_call::*;
pub use array_style::*;
pub use assert_description::*;
pub use assertion_style::*;
pub use block_statement::*;
pub use braced_string_interpolation::*;
pub use class_name::*;
pub use combine_consecutive_issets::*;
pub use constant_condition::*;
pub use constant_name::*;
pub use constant_type::*;
pub use cyclomatic_complexity::*;
pub use disallowed_functions::*;
pub use enum_name::*;
pub use excessive_nesting::*;
pub use excessive_parameter_list::*;
pub use explicit_nullable_param::*;
pub use explicit_octal::*;
pub use function_name::*;
pub use halstead::*;
pub use identity_comparison::*;
pub use interface_name::*;
pub use kan_defect::*;
pub use literal_named_argument::*;
pub use loop_does_not_iterate::*;
pub use lowercase_keyword::*;
pub use lowercase_type_hint::*;
pub use middleware_in_routes::*;
pub use no_alias_function::*;
pub use no_assign_in_condition::*;
pub use no_boolean_flag_parameter::*;
pub use no_boolean_literal_comparison::*;
pub use no_closing_tag::*;
pub use no_debug_symbols::*;
pub use no_else_clause::*;
pub use no_empty::*;
pub use no_empty_catch_clause::*;
pub use no_empty_comment::*;
pub use no_empty_loop::*;
pub use no_error_control_operator::*;
pub use no_eval::*;
pub use no_ffi::*;
pub use no_global::*;
pub use no_goto::*;
pub use no_hash_comment::*;
pub use no_hash_emoji::*;
pub use no_insecure_comparison::*;
pub use no_literal_password::*;
pub use no_multi_assignments::*;
pub use no_noop::*;
pub use no_php_tag_terminator::*;
pub use no_redundant_block::*;
pub use no_redundant_continue::*;
pub use no_redundant_file::*;
pub use no_redundant_final::*;
pub use no_redundant_label::*;
pub use no_redundant_math::*;
pub use no_redundant_method_override::*;
pub use no_redundant_parentheses::*;
pub use no_redundant_string_concat::*;
pub use no_redundant_write_visibility::*;
pub use no_request_all::*;
pub use no_request_variable::*;
pub use no_shell_execute_string::*;
pub use no_short_opening_tag::*;
pub use no_shorthand_ternary::*;
pub use no_sprintf_concat::*;
pub use no_trailing_space::*;
pub use no_underscore_class::*;
pub use no_unsafe_finally::*;
pub use no_void_reference_return::*;
pub use optional_param_order::*;
pub use parameter_type::*;
pub use prefer_anonymous_migration::*;
pub use prefer_interface::*;
pub use prefer_view_array::*;
pub use prefer_while_loop::*;
pub use property_type::*;
pub use psl_array_functions::*;
pub use psl_data_structures::*;
pub use psl_datetime::*;
pub use psl_math_functions::*;
pub use psl_output::*;
pub use psl_randomness_functions::*;
pub use psl_regex_functions::*;
pub use psl_sleep_functions::*;
pub use psl_string_functions::*;
pub use return_type::*;
pub use str_contains::*;
pub use str_starts_with::*;
pub use strict_assertions::*;
pub use strict_behavior::*;
pub use strict_types::*;
pub use tagged_fixme::*;
pub use tagged_todo::*;
pub use tainted_data_to_sink::*;
pub use too_many_enum_cases::*;
pub use too_many_methods::*;
pub use too_many_properties::*;
pub use trait_name::*;
pub use valid_docblock::*;

mod utils;

pub trait Config: Default + DeserializeOwned {
    fn level(&self) -> Level;
}

pub trait LintRule {
    type Config: Config;

    fn meta() -> &'static RuleMeta;

    fn targets() -> &'static [NodeKind];

    #[inline]
    fn is_enabled_for(php_version: PHPVersion, libs: IntegrationSet) -> bool {
        let meta = Self::meta();

        meta.php.includes(php_version) && libs.is_superset_of(meta.requires)
    }

    fn build(settings: RuleSettings<Self::Config>) -> Self;

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>);
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
            pub fn get_all_for(settings: Settings, only: Option<&[String]>) -> Vec<Self> {
                let integrations = IntegrationSet::from_slice(&settings.integrations);
                let mut rules = Vec::new();

                $(
                    let meta = $rule::meta();

                    // If `--only` is used, check if this rule's code is in the list.
                    if let Some(only_codes) = &only {
                        if only_codes.iter().any(|c| c == meta.code) {
                            rules.push(AnyRule::$variant($rule::build(settings.rules.$module)));
                        }
                    } else if settings.rules.$module.enabled && $rule::is_enabled_for(settings.php_version, integrations) {
                        rules.push(AnyRule::$variant($rule::build(settings.rules.$module)));
                    }
                )*

                rules
            }

            #[inline]
            pub fn name(&self) -> &'static str {
                self.meta().name
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
            pub fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>)  {
                match self {
                    $( AnyRule::$variant(r) => r.check(ctx, node), )*
                }
            }
        }
    }
}

define_rules! {
    AmbiguousFunctionCall(ambiguous_function_call @ AmbiguousFunctionCallRule),
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
    NoVoidReferenceReturn(no_void_reference_return @ NoVoidReferenceReturnRule),
    NoUnderscoreClass(no_underscore_class @ NoUnderscoreClassRule),
    NoTrailingSpace(no_trailing_space @ NoTrailingSpaceRule),
    NoRedundantWriteVisibility(no_redundant_write_visibility @ NoRedundantWriteVisibilityRule),
    NoRedundantStringConcat(no_redundant_string_concat @ NoRedundantStringConcatRule),
    NoRedundantParentheses(no_redundant_parentheses @ NoRedundantParenthesesRule),
    NoRedundantMethodOverride(no_redundant_method_override @ NoRedundantMethodOverrideRule),
    NoRedundantMath(no_redundant_math @ NoRedundantMathRule),
    NoRedundantLabel(no_redundant_label @ NoRedundantLabelRule),
    NoRedundantFinal(no_redundant_final @ NoRedundantFinalRule),
    NoRedundantFile(no_redundant_file @ NoRedundantFileRule),
    NoRedundantContinue(no_redundant_continue @ NoRedundantContinueRule),
    NoRedundantBlock(no_redundant_block @ NoRedundantBlockRule),
    NoPhpTagTerminator(no_php_tag_terminator @ NoPhpTagTerminatorRule),
    NoNoop(no_noop @ NoNoopRule),
    NoMultiAssignments(no_multi_assignments @ NoMultiAssignmentsRule),
    NoHashEmoji(no_hash_emoji @ NoHashEmojiRule),
    NoHashComment(no_hash_comment @ NoHashCommentRule),
    NoGoto(no_goto @ NoGotoRule),
    NoGlobal(no_global @ NoGlobalRule),
    NoFfi(no_ffi @ NoFfiRule),
    NoEval(no_eval @ NoEvalRule),
    NoErrorControlOperator(no_error_control_operator @ NoErrorControlOperatorRule),
    NoEmpty(no_empty @ NoEmptyRule),
    NoEmptyLoop(no_empty_loop @ NoEmptyLoopRule),
    NoEmptyComment(no_empty_comment @ NoEmptyCommentRule),
    NoEmptyCatchClause(no_empty_catch_clause @ NoEmptyCatchClauseRule),
    NoElseClause(no_else_clause @ NoElseClauseRule),
    NoClosingTag(no_closing_tag @ NoClosingTagRule),
    NoBooleanLiteralComparison(no_boolean_literal_comparison @ NoBooleanLiteralComparisonRule),
    NoBooleanFlagParameter(no_boolean_flag_parameter @ NoBooleanFlagParameterRule),
    NoAssignInCondition(no_assign_in_condition @ NoAssignInConditionRule),
    NoAliasFunction(no_alias_function @ NoAliasFunctionRule),
    LowercaseTypeHint(lowercase_type_hint @ LowercaseTypeHintRule),
    InterfaceName(interface_name @ InterfaceNameRule),
    IdentityComparison(identity_comparison @ IdentityComparisonRule),
    FunctionName(function_name @ FunctionNameRule),
    ExplicitOctal(explicit_octal @ ExplicitOctalRule),
    ExplicitNullableParam(explicit_nullable_param @ ExplicitNullableParamRule),
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
    ConstantCondition(constant_condition @ ConstantConditionRule),
    NoInsecureComparison(no_insecure_comparison @ NoInsecureComparisonRule),
    NoLiteralPassword(no_literal_password @ NoLiteralPasswordRule),
    TaintedDataToSink(tainted_data_to_sink @ TaintedDataToSinkRule),
    ParameterType(parameter_type @ ParameterTypeRule),
    PropertyType(property_type @ PropertyTypeRule),
    NoUnsafeFinally(no_unsafe_finally @ NoUnsafeFinallyRule),
    StrictAssertions(strict_assertions @ StrictAssertionsRule),
    NoRequestAll(no_request_all @ NoRequestAllRule),
    MiddlewareInRoutes(middleware_in_routes @ MiddlewareInRoutesRule),
}
