use config::ConfigBuilder;
use config::Value;
use config::ValueKind;
use config::builder::BuilderState;
use serde::Deserialize;
use serde::Serialize;

use mago_formatter::settings::*;

use crate::config::ConfigurationEntry;
use crate::error::Error;

/// Configuration options for formatting source code.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatterConfiguration {
    /// A list of patterns to exclude from formatting.
    ///
    /// Defaults to `[]`.
    pub excludes: Vec<String>,

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

    /// Whether to put the `static` keyword before the visibility keyword.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_before_visibility: Option<bool>,

    /// Null type hint style (`null|foo` or `?foo`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub null_type_hint: Option<NullTypeHint>,

    /// Spacing in union/intersection types (`A | B` or `A|B`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_spacing: Option<usize>,

    /// Whether to break a parameter list into multiple lines if it contains a promoted property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub break_promoted_properties_list: Option<bool>,

    /// Whether to add a space before and after the concatenation operator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_concatenation: Option<bool>,

    /// How to format broken method/property chains.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_chain_breaking_style: Option<MethodChainBreakingStyle>,

    /// Whether to add a line before a binary operator or after if it is broken.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_before_binary_operator: Option<bool>,

    /// Whether to sort `use` statements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_uses: Option<bool>,

    /// Whether to separate `use` statements by type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separate_use_types: Option<bool>,

    /// Whether to expand `use` groups.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expand_use_groups: Option<bool>,

    /// Whether to remove the trailing close tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_trailing_close_tag: Option<bool>,

    /// Whether to add a space before the colon in enum backing type hints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_before_enum_backing_type_hint_colon: Option<bool>,

    /// Controls whether to include parentheses around instantiation expressions
    /// when they are followed by a member access operator (`->`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parentheses_around_new_in_member_access: Option<bool>,

    /// Controls whether to include parentheses in `new` expressions, even when no arguments are provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parentheses_in_new_expression: Option<bool>,

    /// Controls whether to include parentheses in `exit` and `die` constructs,
    /// making them resemble function calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parentheses_in_exit_and_die: Option<bool>,

    /// Controls whether to include a space after the `!` operator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_after_not_operator: Option<bool>,
}

impl FormatterConfiguration {
    pub fn get_settings(&self) -> FormatSettings {
        let default = FormatSettings::default();

        FormatSettings {
            print_width: self.print_width.unwrap_or(default.print_width),
            tab_width: self.tab_width.unwrap_or(default.tab_width),
            use_tabs: self.use_tabs.unwrap_or(default.use_tabs),
            end_of_line: self.end_of_line.unwrap_or(default.end_of_line),
            single_quote: self.single_quote.unwrap_or(default.single_quote),
            trailing_comma: self.trailing_comma.unwrap_or(default.trailing_comma),
            space_around_declare_equals: self
                .space_around_declare_equals
                .unwrap_or(default.space_around_declare_equals),
            control_space_parens: self.control_space_parens.unwrap_or(default.control_space_parens),
            closure_brace_style: self.closure_brace_style.unwrap_or(default.closure_brace_style),
            function_brace_style: self.function_brace_style.unwrap_or(default.function_brace_style),
            method_brace_style: self.method_brace_style.unwrap_or(default.method_brace_style),
            classlike_brace_style: self.classlike_brace_style.unwrap_or(default.classlike_brace_style),
            control_brace_style: self.control_brace_style.unwrap_or(default.control_brace_style),
            space_before_closure_params: self
                .space_before_closure_params
                .unwrap_or(default.space_before_closure_params),
            space_after_closure_use: self.space_after_closure_use.unwrap_or(default.space_after_closure_use),
            space_before_arrow_function_params: self
                .space_before_arrow_function_params
                .unwrap_or(default.space_before_arrow_function_params),
            static_before_visibility: self.static_before_visibility.unwrap_or(default.static_before_visibility),
            null_type_hint: self.null_type_hint.unwrap_or(default.null_type_hint),
            type_spacing: self.type_spacing.unwrap_or(default.type_spacing),
            break_promoted_properties_list: self
                .break_promoted_properties_list
                .unwrap_or(default.break_promoted_properties_list),
            space_concatenation: self.space_concatenation.unwrap_or(default.space_concatenation),
            method_chain_breaking_style: self
                .method_chain_breaking_style
                .unwrap_or(default.method_chain_breaking_style),
            line_before_binary_operator: self
                .line_before_binary_operator
                .unwrap_or(default.line_before_binary_operator),
            sort_uses: self.sort_uses.unwrap_or(default.sort_uses),
            separate_use_types: self.separate_use_types.unwrap_or(default.separate_use_types),
            expand_use_groups: self.expand_use_groups.unwrap_or(default.expand_use_groups),
            remove_trailing_close_tag: self.remove_trailing_close_tag.unwrap_or(default.remove_trailing_close_tag),
            space_before_enum_backing_type_hint_colon: self
                .space_before_enum_backing_type_hint_colon
                .unwrap_or(default.space_before_enum_backing_type_hint_colon),
            parentheses_around_new_in_member_access: self
                .parentheses_around_new_in_member_access
                .unwrap_or(default.parentheses_around_new_in_member_access),
            parentheses_in_new_expression: self
                .parentheses_in_new_expression
                .unwrap_or(default.parentheses_in_new_expression),
            parentheses_in_exit_and_die: self
                .parentheses_in_exit_and_die
                .unwrap_or(default.parentheses_in_exit_and_die),
            space_after_not_operator: self.space_after_not_operator.unwrap_or(default.space_after_not_operator),
        }
    }
}

impl ConfigurationEntry for FormatterConfiguration {
    fn configure<St: BuilderState>(self, builder: ConfigBuilder<St>) -> Result<ConfigBuilder<St>, Error> {
        builder.set_default("format.excludes", Value::new(None, ValueKind::Array(vec![]))).map_err(Error::from)
    }
}
