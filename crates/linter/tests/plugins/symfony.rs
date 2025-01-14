use mago_linter::plugin::symfony::rules::quality::interface_should_be_used::InterfaceShouldBeUsed;

use crate::rule_test;

rule_test!(test_interface_should_be_used, InterfaceShouldBeUsed);
