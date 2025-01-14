use mago_linter::plugin::safety::rules::no_error_control_operator::NoErrorControlOperatorRule;
use mago_linter::plugin::safety::rules::no_eval::NoEvalRule;
use mago_linter::plugin::safety::rules::no_ffi::NoFFIRule;
use mago_linter::plugin::safety::rules::no_global::NoGlobalRule;
use mago_linter::plugin::safety::rules::no_request_variable::NoRequestVariableRule;
use mago_linter::plugin::safety::rules::no_shell_execute_string::NoShellExecuteStringRule;
use mago_linter::plugin::safety::rules::no_unsafe_finally::NoUnsafeFinallyRule;

use crate::rule_test;

rule_test!(test_no_error_control_operator, NoErrorControlOperatorRule);
rule_test!(test_no_eval, NoEvalRule);
rule_test!(test_no_ffi, NoFFIRule);
rule_test!(test_no_global, NoGlobalRule);
rule_test!(test_no_request_variable, NoRequestVariableRule);
rule_test!(test_no_shell_execute_string, NoShellExecuteStringRule);
rule_test!(test_no_unsafe_finally, NoUnsafeFinallyRule);
