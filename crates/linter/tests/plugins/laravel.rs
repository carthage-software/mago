use mago_linter::plugin::laravel::rules::best_practices::anonymous_migration::AnonymousMigrationRule;
use mago_linter::plugin::laravel::rules::best_practices::middleware_in_routes::MiddlewareInRoutesRule;
use mago_linter::plugin::laravel::rules::best_practices::view_array_parameter::ViewArrayParameterRule;
use mago_linter::plugin::laravel::rules::safety::no_request_all::NoRequestAllRule;

use crate::rule_test;

rule_test!(test_no_request_all, NoRequestAllRule);
rule_test!(test_anonymous_migration, AnonymousMigrationRule);
rule_test!(test_view_array_parameter, ViewArrayParameterRule);
rule_test!(test_middleware_in_routes, MiddlewareInRoutesRule);
