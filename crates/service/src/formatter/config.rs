use serde::Deserialize;
use serde::Serialize;

use fennec_formatter::settings::*;

/// Configuration options for formatting source code.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormatterConfiguration {
    /// Specify the maximum line length that the printer will wrap on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_width: Option<usize>,

    /// Specify the number of spaces per indentation-level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_width: Option<usize>,

    /// Indent lines with tabs instead of spaces.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_tabs: Option<bool>,

    /// Specify which end-of-line characters to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_of_line: Option<EndOfLine>,

    /// Use single quotes instead of double quotes for strings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_quote: Option<bool>,

    /// Enable or disable trailing commas in multi-line syntactic structures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_comma: Option<bool>,

    /// Add spaces around the `=` in declare statements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_around_declare_equals: Option<bool>,

    /// Keyword casing (e.g., lowercase, uppercase, camelCase).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword_case: Option<CasingStyle>,

    /// Casting operator for strings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_cast: Option<StringCastOperator>,

    /// Casting operator for floats.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub float_cast: Option<FloatCastOperator>,

    /// Casting operator for booleans.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bool_cast: Option<BoolCastOperator>,

    /// Casting operator for integers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int_cast: Option<IntCastOperator>,

    /// Leave casting operators as is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leave_casts_as_is: Option<bool>,

    /// Include `?>` in files containing only PHP code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_closing_tag: Option<bool>,

    /// Blank line after the opening PHP tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blank_line_after_open_tag: Option<bool>,

    /// Controls whether a single breaking argument (e.g., an array or closure) is inlined within the enclosing parentheses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_single_breaking_argument: Option<bool>,

    /// Controls whether a single breaking attribute is inlined within the `#[` and `]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_single_attribute: Option<bool>,

    /// In a control structure expression, is there a space after the opening parenthesis
    ///  and a space before the closing parenthesis?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_space_parens: Option<bool>,

    /// Brace style for closures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closure_brace_style: Option<BraceStyle>,

    /// Brace style for function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_brace_style: Option<BraceStyle>,

    /// Brace style for methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_brace_style: Option<BraceStyle>,

    /// Brace style for class-like structures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classlike_brace_style: Option<BraceStyle>,

    /// Brace style for control structures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_brace_style: Option<BraceStyle>,

    /// Space between the `function` keyword and the opening parenthesis in closure declarations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_before_closure_params: Option<bool>,

    /// Space between the `use` keyword and the opening parenthesis in closure declarations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_after_closure_use: Option<bool>,

    /// Space between the `fn` keyword and the opening parenthesis in arrow function declarations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_before_arrow_function_params: Option<bool>,

    /// Space between function name and opening parenthesis in calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_after_function_name: Option<bool>,

    /// Order of `static` and visibility in method declarations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_visibility_order: Option<StaticVisibilityOrder>,

    /// Require parentheses around class instantiations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_instantiation_parens: Option<bool>,

    /// Sort methods alphabetically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_methods: Option<bool>,

    /// Sort properties alphabetically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_properties: Option<bool>,

    /// Sort enum cases alphabetically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_enum_cases: Option<bool>,

    /// Sort class-like constants alphabetically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_classlike_constants: Option<bool>,

    /// Ensure constructor is the first method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constructor_first: Option<bool>,

    /// Ensure destructor is the last method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructor_last: Option<bool>,

    /// Static methods come before non-static methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_methods_first: Option<bool>,

    /// Static properties come before non-static properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_properties_first: Option<bool>,

    /// Split grouped `use` statements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split_use_statements: Option<bool>,

    /// List style (`[a, b]` or `list(a, b)`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_style: Option<ListStyle>,

    /// Null type hint style (`null|foo` or `?foo`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub null_type_hint: Option<NullTypeHint>,

    /// Spacing around binary operators.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_op_spacing: Option<usize>,

    /// Replace `<>` with `!=`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_angle_not_equals: Option<bool>,

    /// Spacing in union/intersection types (`A | B` or `A|B`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_spacing: Option<usize>,

    /// Split constants and properties into separate statements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split_multi_declare: Option<bool>,

    /// The minimum length of a method call chain that triggers line-breaking formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_chain_break_threshold: Option<usize>,

    /// Whether to break a parameter list into multiple lines if it contains a promoted property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub break_promoted_properties_list: Option<bool>,

    /// Whether to add a space before and after the concatenation operator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_concatenation: Option<bool>,

    /// Whether to preserve argument lists that are already broken into multiple lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_broken_argument_lists: Option<bool>,

    /// Whether to inline a single attribute group in a parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_single_attribute_group: Option<bool>,

    /// Whether to preserve newlines between attribute groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_attribute_group_newlines: Option<bool>,

    /// Preserve existing newlines in parameter lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_multiline_parameters: Option<bool>,

    /// Whether to preserve binary operations that are already broken into multiple lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_multiline_binary_operations: Option<bool>,

    /// How to format broken method/property chains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_chain_breaking_style: Option<MethodChainBreakingStyle>,
}

impl FormatterConfiguration {
    pub fn get_settings(&self) -> FormatSettings {
        let d = FormatSettings::default();

        FormatSettings {
            print_width: self.print_width.unwrap_or(d.print_width),
            tab_width: self.tab_width.unwrap_or(d.tab_width),
            use_tabs: self.use_tabs.unwrap_or(d.use_tabs),
            end_of_line: self.end_of_line.unwrap_or(d.end_of_line),
            single_quote: self.single_quote.unwrap_or(d.single_quote),
            trailing_comma: self.trailing_comma.unwrap_or(d.trailing_comma),
            space_around_declare_equals: self.space_around_declare_equals.unwrap_or(d.space_around_declare_equals),
            keyword_case: self.keyword_case.unwrap_or(d.keyword_case),
            string_cast: self.string_cast.unwrap_or(d.string_cast),
            float_cast: self.float_cast.unwrap_or(d.float_cast),
            bool_cast: self.bool_cast.unwrap_or(d.bool_cast),
            int_cast: self.int_cast.unwrap_or(d.int_cast),
            leave_casts_as_is: self.leave_casts_as_is.unwrap_or(d.leave_casts_as_is),
            include_closing_tag: self.include_closing_tag.unwrap_or(d.include_closing_tag),
            blank_line_after_open_tag: self.blank_line_after_open_tag.unwrap_or(d.blank_line_after_open_tag),
            inline_single_breaking_argument: self
                .inline_single_breaking_argument
                .unwrap_or(d.inline_single_breaking_argument),
            inline_single_attribute: self.inline_single_attribute.unwrap_or(d.inline_single_attribute),
            control_space_parens: self.control_space_parens.unwrap_or(d.control_space_parens),
            closure_brace_style: self.closure_brace_style.unwrap_or(d.closure_brace_style),
            function_brace_style: self.function_brace_style.unwrap_or(d.function_brace_style),
            method_brace_style: self.method_brace_style.unwrap_or(d.method_brace_style),
            classlike_brace_style: self.classlike_brace_style.unwrap_or(d.classlike_brace_style),
            control_brace_style: self.control_brace_style.unwrap_or(d.control_brace_style),
            space_after_function_name: self.space_after_function_name.unwrap_or(d.space_after_function_name),
            space_before_closure_params: self.space_before_closure_params.unwrap_or(d.space_before_closure_params),
            space_after_closure_use: self.space_after_closure_use.unwrap_or(d.space_after_closure_use),
            space_before_arrow_function_params: self
                .space_before_arrow_function_params
                .unwrap_or(d.space_before_arrow_function_params),
            static_visibility_order: self.static_visibility_order.unwrap_or(d.static_visibility_order),
            require_instantiation_parens: self.require_instantiation_parens.unwrap_or(d.require_instantiation_parens),
            sort_methods: self.sort_methods.unwrap_or(d.sort_methods),
            sort_properties: self.sort_properties.unwrap_or(d.sort_properties),
            sort_enum_cases: self.sort_enum_cases.unwrap_or(d.sort_enum_cases),
            sort_classlike_constants: self.sort_classlike_constants.unwrap_or(d.sort_classlike_constants),
            constructor_first: self.constructor_first.unwrap_or(d.constructor_first),
            destructor_last: self.destructor_last.unwrap_or(d.destructor_last),
            static_methods_first: self.static_methods_first.unwrap_or(d.static_methods_first),
            static_properties_first: self.static_properties_first.unwrap_or(d.static_properties_first),
            split_use_statements: self.split_use_statements.unwrap_or(d.split_use_statements),
            list_style: self.list_style.unwrap_or(d.list_style),
            null_type_hint: self.null_type_hint.unwrap_or(d.null_type_hint),
            binary_op_spacing: self.binary_op_spacing.unwrap_or(d.binary_op_spacing),
            replace_angle_not_equals: self.replace_angle_not_equals.unwrap_or(d.replace_angle_not_equals),
            type_spacing: self.type_spacing.unwrap_or(d.type_spacing),
            split_multi_declare: self.split_multi_declare.unwrap_or(d.split_multi_declare),
            method_chain_break_threshold: self.method_chain_break_threshold.unwrap_or(d.method_chain_break_threshold),
            break_promoted_properties_list: self
                .break_promoted_properties_list
                .unwrap_or(d.break_promoted_properties_list),
            space_concatenation: self.space_concatenation.unwrap_or(d.space_concatenation),
            preserve_broken_argument_lists: self
                .preserve_broken_argument_lists
                .unwrap_or(d.preserve_broken_argument_lists),
            inline_single_attribute_group: self
                .inline_single_attribute_group
                .unwrap_or(d.inline_single_attribute_group),
            preserve_attribute_group_newlines: self
                .preserve_attribute_group_newlines
                .unwrap_or(d.preserve_attribute_group_newlines),
            preserve_multiline_parameters: self
                .preserve_multiline_parameters
                .unwrap_or(d.preserve_multiline_parameters),
            preserve_multiline_binary_operations: self
                .preserve_multiline_binary_operations
                .unwrap_or(d.preserve_multiline_binary_operations),
            method_chain_breaking_style: self.method_chain_breaking_style.unwrap_or(d.method_chain_breaking_style),
        }
    }
}
