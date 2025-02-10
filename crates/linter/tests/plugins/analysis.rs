use mago_linter::plugin::analysis::rules::instantiation::InstantiationRule;
use mago_linter::plugin::analysis::rules::undefined_constant_or_case::UndefinedConstantOrCaseRule;
use mago_linter::plugin::analysis::rules::undefined_function_or_method::UndefinedFunctionOrMethodRule;

use crate::rule_test;

rule_test!(test_instantiation, InstantiationRule);
rule_test!(test_undefined_constant_or_case, UndefinedConstantOrCaseRule);
rule_test!(test_undefined_function_or_method, UndefinedFunctionOrMethodRule);
