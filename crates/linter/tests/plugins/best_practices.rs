use mago_linter::plugin::best_practices::rules::combine_consecutive_issets::CombineConsecutiveIssetsRule;
use mago_linter::plugin::best_practices::rules::disallowed_functions::DisallowedFunctionsRule;
use mago_linter::plugin::best_practices::rules::excessive_nesting::ExcessiveNesting;
use mago_linter::plugin::best_practices::rules::loop_does_not_iterate::LoopDoesNotIterateRule;
use mago_linter::plugin::best_practices::rules::no_debug_symbols::NoDebugSymbolsRule;
use mago_linter::plugin::best_practices::rules::no_empty_loop::NoEmptyLoopRule;
use mago_linter::plugin::best_practices::rules::no_goto::NoGotoRule;
use mago_linter::plugin::best_practices::rules::no_hash_emoji::NoHashEmojiRule;
use mago_linter::plugin::best_practices::rules::no_multi_assignments::NoMultiAssignmentsRule;
use mago_linter::plugin::best_practices::rules::no_unused_parameter::NoUnusedParameterRule;
use mago_linter::plugin::best_practices::rules::use_while_instead_of_for::UseWhileInsteadOfForRule;

use crate::rule_test;

rule_test!(test_combine_consecutive_issets, CombineConsecutiveIssetsRule);
rule_test!(test_disallowed_functions, DisallowedFunctionsRule);
rule_test!(test_excessive_nesting, ExcessiveNesting);
rule_test!(test_loop_does_not_iterate, LoopDoesNotIterateRule);
rule_test!(test_no_debug_symbols, NoDebugSymbolsRule);
rule_test!(test_no_empty_loop, NoEmptyLoopRule);
rule_test!(test_no_goto, NoGotoRule);
rule_test!(test_no_hash_emoji, NoHashEmojiRule);
rule_test!(test_no_multi_assignments, NoMultiAssignmentsRule);
rule_test!(test_no_unused_parameter, NoUnusedParameterRule);
rule_test!(test_use_while_instead_of_for, UseWhileInsteadOfForRule);
