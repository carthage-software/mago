use mago_formatter::Formatter;
use mago_formatter::settings::FormatSettings;
use mago_interner::ThreadedInterner;
use mago_php_version::PHPVersion;

#[macro_export]
macro_rules! test_case {
    ($name:ident) => {
        $crate::test_case!($name, PHPVersion::PHP84);
    };
    ($name:ident, $version:expr) => {
        #[test]
        pub fn $name() {
            let code = include_str!(concat!("./cases/", stringify!($name), "/before.php"));
            let expected = include_str!(concat!("./cases/", stringify!($name), "/after.php"));
            let settings = include!(concat!("./cases/", stringify!($name), "/settings.inc"));

            let interner = ThreadedInterner::new();
            let formatter = Formatter::new(&interner, $version, settings);

            let formatted_code = formatter.format_code("code.php", code.as_ref()).unwrap();
            pretty_assertions::assert_eq!(expected, formatted_code, "Formatted code does not match expected");

            let reformatted_code = formatter.format_code("formatted_code.php", &formatted_code).unwrap();
            pretty_assertions::assert_eq!(expected, reformatted_code, "Reformatted code does not match expected");
        }
    };
}

// Test cases
test_case!(leading_comment_with_missing_prefix);
test_case!(dangling_block_comments);
test_case!(opening_tag_trailing_comments);
test_case!(callee_needs_parens);
test_case!(php83_instantiation_with_member_access_parentheses, PHPVersion::PHP83);
test_case!(php84_instantiation_with_member_access_parentheses, PHPVersion::PHP84);
test_case!(php83_instantiation_with_member_access_parentheses_disabled, PHPVersion::PHP83);
test_case!(php84_instantiation_with_member_access_parentheses_disabled, PHPVersion::PHP84);
test_case!(expand_last_argument);
test_case!(expand_first_argument);
test_case!(hug_new_expression);
test_case!(hug_new_expression_with_simple_args);
test_case!(hug_last_new_expression_with_named_args);
test_case!(assignments);
test_case!(assignments_chain);
test_case!(assignment_breaks_after_operator);
test_case!(conditional_assignment);
test_case!(conditional_assignment_wide);
test_case!(conditional_assignment_narrow);
test_case!(conditional_assignment_super_narrow);
test_case!(logical_operations_within_parens);
test_case!(simple_binaryish_operators);
test_case!(multiple_concat_operations_in_array);
test_case!(binary_operand_needs_parens);
test_case!(binary_ops_wrapping);
test_case!(parens_around_constructs);
test_case!(interpolated_strings_vars);
test_case!(closure_creation);
test_case!(fluid_member_access_chain);
test_case!(member_access_chain_in_conditional);
test_case!(short_member_access_chain);
test_case!(use_sorting);
test_case!(use_sorting_with_separation);
test_case!(use_sorting_with_expansion);
test_case!(use_sorting_separation_expansion);
test_case!(use_no_change);
test_case!(use_mixed_use_list);
test_case!(use_mixed_use_list_expanded);
test_case!(docs_before_use_are_preserved);
test_case!(mixed_expressions);
test_case!(inline_html);
test_case!(inline_php);
test_case!(inline_html_alignment);
test_case!(inline_echo);
test_case!(parameter_attributes);
test_case!(space_before_enum_backing_type_colon);
test_case!(no_space_before_enum_backing_type_colon);
test_case!(closing_tag_removed);
test_case!(closing_tag_preserved);
test_case!(single_quote_string);
test_case!(double_quote_string);
test_case!(inline_if_statement);
test_case!(top_level_inline_if);
test_case!(inline_control_structures);
test_case!(aligned_inline_control_structures);
test_case!(html_template);
test_case!(complex_html_template);
test_case!(attributes);
test_case!(keyword_as_method_name);
test_case!(parens_around_closure);
test_case!(parens);
test_case!(use_typed_use_list_expanded);
test_case!(use_nested_namespace_expanded);
test_case!(adds_empty_line_after_use);
test_case!(leaves_single_empty_line_after_use);
test_case!(nesting_wrap);
test_case!(nesting_wrap_wide);
test_case!(nesting_wrap_more_narrow);
test_case!(nesting_wrap_narrow);
test_case!(nesting_wrap_super_narrow);
test_case!(awaitable);
test_case!(argument_list_comments);
test_case!(space_after_not_operator);
test_case!(breaking_named_arguments);
test_case!(break_fn_args);
test_case!(member_access_chain);
test_case!(return_wrapping);
test_case!(arrow_return);
test_case!(match_breaking);
test_case!(array_alignment);
test_case!(binary_alignment);
test_case!(binary_alignment_before_op);
test_case!(chain_comments);
test_case!(literal_concat_parens);
test_case!(preserve_breaking_member_access_chain);
test_case!(preserve_breaking_member_access_chain_disabled);
test_case!(preserve_breaking_argument_list);
test_case!(preserve_breaking_argument_list_disabled);
test_case!(preserve_breaking_array_like);
test_case!(preserve_breaking_array_like_disabled);
test_case!(preserve_breaking_parameter_list);
test_case!(preserve_breaking_parameter_list_disabled);
test_case!(preserve_breaking_attribute_list);
test_case!(preserve_breaking_attribute_list_disabled);
test_case!(preserve_breaking_conditional_expression);
test_case!(preserve_breaking_conditional_expression_disabled);
test_case!(hooks_always_break);
test_case!(comments_are_preserved);
test_case!(array_comment);
test_case!(spacing_options);
test_case!(spacing_options_flipped);

// GitHub issue test cases
test_case!(issue_122);
test_case!(issue_123);
test_case!(issue_128);
test_case!(issue_130);
test_case!(issue_138);
test_case!(issue_149);
test_case!(issue_150);
test_case!(issue_151);
