use mago_linter::plugin::phpunit::rules::consistency::assertions_style::AssertionsStyleRule;
use mago_linter::plugin::phpunit::rules::strictness::strict_assertions::StrictAssertionsRule;

use crate::rule_test;

rule_test!(test_assertions_style, AssertionsStyleRule);
rule_test!(test_strict_assertions, StrictAssertionsRule);
