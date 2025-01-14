use mago_linter::plugin::migration::rules::php80::str_contains::StrContainsRule;
use mago_linter::plugin::migration::rules::php80::str_starts_with::StrStartsWithRule;
use mago_linter::plugin::migration::rules::php81::explicit_octal_notation::ExplicitOctalNotationRule;
use mago_linter::plugin::migration::rules::php82::readonly_class_promotion::ReadonlyClassPromotionRule;

use crate::rule_test;

rule_test!(test_str_starts_with, StrStartsWithRule);
rule_test!(test_str_contains, StrContainsRule);
rule_test!(test_explicit_octal_notation, ExplicitOctalNotationRule);
rule_test!(test_readonly_class_promotion, ReadonlyClassPromotionRule);
