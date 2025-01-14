use mago_linter::plugin::laravel::rules::safety::no_request_all::NoRequestAllRule;

use crate::rule_test;

rule_test!(test_no_request_all, NoRequestAllRule);
