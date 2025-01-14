use mago_linter::plugin::analysis::rules::inheritance::InheritanceRule;
use mago_linter::plugin::analysis::rules::instantiation::InstantiationRule;
use mago_linter::plugin::analysis::rules::undefined_constant::UndefinedConstantRule;
use mago_linter::plugin::analysis::rules::undefined_function::UndefinedFunctionRule;

use crate::rule_test;

rule_test!(test_inheritance, InheritanceRule);
rule_test!(test_instantiation, InstantiationRule);
rule_test!(test_undefined_constant, UndefinedConstantRule);
rule_test!(test_undefined_function, UndefinedFunctionRule);
