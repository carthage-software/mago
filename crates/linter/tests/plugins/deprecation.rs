use mago_linter::plugin::deprecation::rules::php80::optional_parameter_before_required::OptionalParameterBeforeRequiredRule;
use mago_linter::plugin::deprecation::rules::php82::return_by_reference_from_void_function::ReturnByReferenceFromVoidFunctionRule;
use mago_linter::plugin::deprecation::rules::php84::implicitly_nullable_parameter::ImplicitlyNullableParameterRule;
use mago_linter::plugin::deprecation::rules::php84::underscore_classname::UnderscoreClassNameRule;

use crate::rule_test;

rule_test!(test_optional_parameter_before_required, OptionalParameterBeforeRequiredRule);
rule_test!(test_return_by_reference_from_void_function, ReturnByReferenceFromVoidFunctionRule);
rule_test!(test_implicitly_nullable_parameter, ImplicitlyNullableParameterRule);
rule_test!(test_underscore_classname, UnderscoreClassNameRule);
