use mago_linter::plugin::naming::rules::class::ClassRule;
use mago_linter::plugin::naming::rules::constant::ConstantRule;
use mago_linter::plugin::naming::rules::function::FunctionRule;
use mago_linter::plugin::naming::rules::interface::InterfaceRule;
use mago_linter::plugin::naming::rules::r#enum::EnumRule;
use mago_linter::plugin::naming::rules::r#trait::TraitRule;

use crate::rule_test;

rule_test!(test_class, ClassRule);
rule_test!(test_constant, ConstantRule);
rule_test!(test_enum, EnumRule);
rule_test!(test_function, FunctionRule);
rule_test!(test_interface, InterfaceRule);
rule_test!(test_trait, TraitRule);
