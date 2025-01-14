use mago_linter::plugin::consistency::rules::array_syntax::ArraySyntaxRule;
use mago_linter::plugin::consistency::rules::lowercase_hint::LowercaseHintRule;
use mago_linter::plugin::consistency::rules::lowercase_keyword::LowercaseKeywordRule;
use mago_linter::plugin::consistency::rules::no_function_aliases::NoFunctionAliasesRule;
use mago_linter::plugin::consistency::rules::no_tag_pair_terminator::NoTagPairTerminatorRule;
use mago_linter::plugin::consistency::rules::require_block_statement_body::RequireBlockStatementBodyRule;

use crate::rule_test;

rule_test!(test_array_syntax, ArraySyntaxRule);
rule_test!(test_lowercase_hint, LowercaseHintRule);
rule_test!(test_lowercase_keyword, LowercaseKeywordRule);
rule_test!(test_no_function_aliases, NoFunctionAliasesRule);
rule_test!(test_no_tag_pair_terminator, NoTagPairTerminatorRule);
rule_test!(test_require_block_statement_body, RequireBlockStatementBodyRule);
