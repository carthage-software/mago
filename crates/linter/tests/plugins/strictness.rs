use mago_linter::plugin::strictness::rules::missing_assert_description::MissingAssertDescriptionRule;
use mago_linter::plugin::strictness::rules::no_assignment_in_condition::NoAssignmentInConditionRule;
use mago_linter::plugin::strictness::rules::no_elvis_operator::NoElvisOperator;
use mago_linter::plugin::strictness::rules::no_empty_construct::NoEmptyConstruct;
use mago_linter::plugin::strictness::rules::require_constant_type::RequireConstantTypeRule;
use mago_linter::plugin::strictness::rules::require_identity_comparison::RequireIdentityComparisonRule;
use mago_linter::plugin::strictness::rules::require_parameter_type::RequireParameterTypeRule;
use mago_linter::plugin::strictness::rules::require_property_type::RequirePropertyTypeRule;
use mago_linter::plugin::strictness::rules::require_return_type::RequireReturnTypeRule;
use mago_linter::plugin::strictness::rules::require_strict_behavior::RequireStrictBehavior;
use mago_linter::plugin::strictness::rules::require_strict_types::RequireStrictTypesRule;

use crate::rule_test;

rule_test!(test_missing_assert_description, MissingAssertDescriptionRule);
rule_test!(test_no_assignment_in_condition, NoAssignmentInConditionRule);
rule_test!(test_no_elvis_operator, NoElvisOperator);
rule_test!(test_no_empty_construct, NoEmptyConstruct);
rule_test!(test_require_constant_type, RequireConstantTypeRule);
rule_test!(test_require_identity_comparison, RequireIdentityComparisonRule);
rule_test!(test_require_parameter_type, RequireParameterTypeRule);
rule_test!(test_require_property_type, RequirePropertyTypeRule);
rule_test!(test_require_return_type, RequireReturnTypeRule);
rule_test!(test_require_strict_behavior, RequireStrictBehavior);
rule_test!(test_require_strict_types, RequireStrictTypesRule);
