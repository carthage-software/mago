use mago_linter::plugin::psl::rules::array_functions::ArrayFunctionsRule;
use mago_linter::plugin::psl::rules::data_structures::DataStructuresRule;
use mago_linter::plugin::psl::rules::datetime::DateTimeRule;
use mago_linter::plugin::psl::rules::math_functions::MathFunctionsRule;
use mago_linter::plugin::psl::rules::output::OutputRule;
use mago_linter::plugin::psl::rules::randomness_functions::RandomnessFunctionsRule;
use mago_linter::plugin::psl::rules::regex_functions::RegexFunctionsRule;
use mago_linter::plugin::psl::rules::sleep_functions::SleepFunctionsRule;
use mago_linter::plugin::psl::rules::string_functions::StringFunctionsRule;

use crate::rule_test;

rule_test!(test_array_functions, ArrayFunctionsRule);
rule_test!(test_data_structures, DataStructuresRule);
rule_test!(test_datetime, DateTimeRule);
rule_test!(test_math_functions, MathFunctionsRule);
rule_test!(test_output, OutputRule);
rule_test!(test_randomness_functions, RandomnessFunctionsRule);
rule_test!(test_regex_functions, RegexFunctionsRule);
rule_test!(test_sleep_functions, SleepFunctionsRule);
rule_test!(test_string_functions, StringFunctionsRule);
