use std::fs;

mod framework;

/// A macro to automatically generate a test case from a corresponding PHP file.
///
/// # Variants
///
/// - `test_case!(test_name)`: Creates a test using default settings.
/// - `test_case!(test_name, settings_expression)`: Creates a test with custom settings.
///
/// For a given test name, e.g., `my_test`, this macro will:
///
/// 1. Create a test function named `my_test`.
/// 2. Read the content from the file `cases/my_test.php`.
/// 3. Create and run a `TestCase` with that content and the specified settings.
macro_rules! test_case {
    ($test_name:ident, $settings:expr) => {
        #[test]
        fn $test_name() {
            let content = include_str!(concat!("cases/", stringify!($test_name), ".php"));
            $crate::framework::TestCase::new(stringify!($test_name), content).settings($settings).run();
        }
    };
    ($test_name:ident) => {
        #[test]
        fn $test_name() {
            let content = include_str!(concat!("cases/", stringify!($test_name), ".php"));
            $crate::framework::TestCase::new(stringify!($test_name), content).run();
        }
    };
}

test_case!(accessing_undefined_class_constant);
test_case!(argument_count);
test_case!(array_list_reconciliation);
test_case!(array_shape_fields);
test_case!(assert_generic_array_key_is_array_key);
test_case!(bare_identifier_in_array_access);
test_case!(class_like_constant_access);
test_case!(collection_types);
test_case!(condition_is_too_complex);
test_case!(conditional_if_else);
test_case!(conditional_return_resolved_to_left);
test_case!(conditional_return_resolved_to_right);
test_case!(conditional_return_with_assignment_in_condition);
test_case!(const_array_key);
test_case!(docblock_type_narrowing);
test_case!(docblock_type_parsing_verification);
test_case!(empty_switch);
test_case!(generic_shape_coercion);
test_case!(int_or_float);
test_case!(integer_range_reconciliation);
test_case!(integer_reconciliation);
test_case!(isset_and_nullable_access_assertions);
test_case!(iterable_count);
test_case!(iterable_reconciliation);
test_case!(non_empty_string_magic_constant);
test_case!(numeric_reconciliation);
test_case!(priority_queue_implementation);
test_case!(psl_integration);
test_case!(reconcile_array_index_type);
test_case!(reconcile_empty_string);
test_case!(reconcile_non_empty_string);
test_case!(reconcile_properties);
test_case!(reconciling_generic_parameter);
test_case!(recursive_templates);
test_case!(resource_reconciliation);
test_case!(scalar_types_reconciliation);
test_case!(string_reconciliation);
test_case!(switch_complex_logic);
test_case!(switch_default_only);
test_case!(switch_empty_case);
test_case!(switch_fall_through);
test_case!(switch_mixed_fall_through);
test_case!(switch_multiple_cases);
test_case!(switch_no_break);
test_case!(switch_simple_break);
test_case!(switch_statement);
test_case!(switch_string_subject);
test_case!(switch_with_return);
test_case!(type_guard_followed_by_redundant_check);
test_case!(type_narrowing_and_assertions);
test_case!(unspecified_callable_or_closure);
test_case!(untemplated_generic_parameters);
test_case!(untyped_callable_parameter);
test_case!(yield_array_value);
test_case!(yield_from_generator);
test_case!(yield_from_invalid_key);
test_case!(yield_from_invalid_type);
test_case!(yield_from_non_iterable);
test_case!(yield_global_scope);
test_case!(yield_invalid_key);
test_case!(yield_invalid_value);
test_case!(yield_merge_iterables);
test_case!(switch_always_matching_case);
test_case!(switch_case_after_default_is_unreachable);
test_case!(switch_logically_unreachable_case);
test_case!(switch_on_literal);
test_case!(switch_redundant_condition);
test_case!(conditional_always_truthy);
test_case!(conditional_always_falsy);
test_case!(conditional_mixed_types);
test_case!(conditional_with_assignment);
test_case!(conditional_nested);
test_case!(elvis_operator_with_null);
test_case!(elvis_operator_with_falsy_string);
test_case!(short_ternary_with_truthy);
test_case!(short_ternary_with_falsy);
test_case!(conditional_type_narrowing);
test_case!(type_is_not_narrowed_by_nested_conditional_exit);
test_case!(dynamic_array_key_in_string_interpolation);
test_case!(big_numbers);
test_case!(numeric_non_lowercase_string);
test_case!(not_equal_negation);
test_case!(overload_resolution_for_callable);
test_case!(fully_qualified_use_prefix);
test_case!(extends_inheritance_happy_path);
test_case!(extends_inheritance_error_path);
test_case!(implements_inheritance_happy_path);
test_case!(implements_inheritance_error_path);
test_case!(use_inheritance_happy_path);
test_case!(use_inheritance_error_path);
test_case!(templated_trait_use);
test_case!(template_parameter_sanity_check);
test_case!(calling_trait_required_method);
test_case!(useless_statements);
test_case!(complex_template_constraint);
test_case!(docblock_var_override);
test_case!(isset_refinement_on_object_properties);
test_case!(array_shape_reconciliation);
test_case!(retain_possibly_undefined_in_array_access);
test_case!(static_return_generic_override);
test_case!(default_parameter_inference);
test_case!(infer_class_string_on_generic_object);
test_case!(redefined_loop_variables);
test_case!(keyed_array_list_inference);
test_case!(require_implements_inherited);
test_case!(readonly_proptected_set);
test_case!(value_of_enum_resolution);
test_case!(assert_or_type);
test_case!(infere_closure_parameter_type);
test_case!(negated_union_type);
test_case!(reference_constraint_violation);
test_case!(unreferencable_expression);
test_case!(pass_by_ref);
test_case!(return_by_ref);
test_case!(array_refs);
test_case!(ref_constraint_conflict);
test_case!(match_not_exhaustive);
test_case!(match_expression);
test_case!(match_arm_reaching);
test_case!(properties_added_to_context);
test_case!(throwing_exceptions);
test_case!(reference_reused_from_confusing_scope);
test_case!(reconcile_literl_class_string);
test_case!(class_string_is_never_equal_to_literal_string);
test_case!(narrow_class_string_match);
test_case!(narrow_non_final_class_string_match);
test_case!(expand_class_constant_type);
test_case!(iterator_to_array);
test_case!(arrow_function_inherits_method_templates);
test_case!(all_paths_return_value);
test_case!(reconcile_scalars);
test_case!(static_anonymous_class);
test_case!(private_static_method);
test_case!(property_docblock);

// Github Issues
test_case!(issue_275);
test_case!(issue_306);
test_case!(issue_355);
test_case!(issue_357);
test_case!(issue_358);
test_case!(issue_359);
test_case!(issue_360);
test_case!(issue_361);
test_case!(issue_362);
test_case!(issue_366);
test_case!(issue_368);
test_case!(issue_388);
test_case!(issue_390);
test_case!(issue_391);
test_case!(issue_393);
test_case!(issue_396);
test_case!(issue_400);
test_case!(issue_415);
test_case!(issue_417);

#[test]
fn test_all_test_cases_are_ran() {
    let test_case_file = include_str!("mod.rs");
    let test_cases_dir = fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cases")).unwrap();

    for entry in test_cases_dir {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }

        let file_name = path.file_stem().unwrap().to_str().unwrap();
        assert!(
            test_case_file.contains(&format!("test_case!({})", file_name)),
            "File '{}' was not found as a test case",
            file_name
        );
    }
}
