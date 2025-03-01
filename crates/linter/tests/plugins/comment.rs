use mago_linter::plugin::comment::rules::docblock_syntax::DocblockSyntaxRule;
use mago_linter::plugin::comment::rules::no_empty_comments::NoEmptyCommentsRule;
use mago_linter::plugin::comment::rules::no_shell_style::NoShellStyleRule;
use mago_linter::plugin::comment::rules::no_trailing_whitespace::NoTrailingWhitespaceRule;
use mago_linter::plugin::comment::rules::no_untagged_fixme::NoUntaggedFixmeRule;
use mago_linter::plugin::comment::rules::no_untagged_todo::NoUntaggedTodoRule;
use mago_linter::plugin::comment::rules::use_expect_instead_of_ignore::UseExpectInsteadOfIgnoreRule;

use crate::rule_test;

rule_test!(test_no_untagged_todo, NoUntaggedTodoRule);
rule_test!(test_no_empty_comments, NoEmptyCommentsRule);
rule_test!(test_no_untagged_fixme, NoUntaggedFixmeRule);
rule_test!(test_no_shell_style, NoShellStyleRule);
rule_test!(test_no_trailing_whitespace, NoTrailingWhitespaceRule);
rule_test!(test_docblock_syntax, DocblockSyntaxRule);
rule_test!(test_use_expect_instead_of_ignore, UseExpectInsteadOfIgnoreRule);
