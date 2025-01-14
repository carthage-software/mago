use mago_linter::plugin::redundancy::rules::redundant_block::RedundantBlockRule;
use mago_linter::plugin::redundancy::rules::redundant_closing_tag::RedudnantClosingTagRule;
use mago_linter::plugin::redundancy::rules::redundant_continue::RedundantContinueRule;
use mago_linter::plugin::redundancy::rules::redundant_final_method_modifier::RedundantFinalMethodModifierRule;
use mago_linter::plugin::redundancy::rules::redundant_if_statement::RedundantIfStatementRule;
use mago_linter::plugin::redundancy::rules::redundant_label::RedundantLabelRule;
use mago_linter::plugin::redundancy::rules::redundant_method_override::RedundantMethodOverrideRule;
use mago_linter::plugin::redundancy::rules::redundant_noop::RedundantNoopRule;
use mago_linter::plugin::redundancy::rules::redundant_parentheses::RedundantParenthesesRule;
use mago_linter::plugin::redundancy::rules::redundant_string_concat::RedundantStringConcatRule;

use crate::rule_test;

rule_test!(test_redundant_block, RedundantBlockRule);
rule_test!(test_redundant_closing_tag, RedudnantClosingTagRule);
rule_test!(test_redundant_continue, RedundantContinueRule);
rule_test!(test_redundant_final_method_modifier, RedundantFinalMethodModifierRule);
rule_test!(test_redundant_if_statement, RedundantIfStatementRule);
rule_test!(test_redundant_label, RedundantLabelRule);
rule_test!(test_redundant_method_override, RedundantMethodOverrideRule);
rule_test!(test_redundant_noop, RedundantNoopRule);
rule_test!(test_redundant_parentheses, RedundantParenthesesRule);
rule_test!(test_redundant_string_concat, RedundantStringConcatRule);
