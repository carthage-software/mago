use config::builder::BuilderState;
use config::ConfigBuilder;
use serde::Deserialize;
use serde::Serialize;

use mago_formatter::settings::*;

use crate::config::ConfigurationEntry;
use crate::error::Error;

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

    /// Blank line after the opening PHP tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blank_line_after_open_tag: Option<bool>,

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

    /// Spacing around binary operators.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_op_spacing: Option<usize>,

    /// Spacing in union/intersection types (`A | B` or `A|B`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_spacing: Option<usize>,

    /// The minimum length of a method call chain that triggers line-breaking formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_chain_break_threshold: Option<usize>,

    /// Whether to break a parameter list into multiple lines if it contains a promoted property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub break_promoted_properties_list: Option<bool>,

    /// Whether to add a space before and after the concatenation operator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_concatenation: Option<bool>,

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
            blank_line_after_open_tag: self.blank_line_after_open_tag.unwrap_or(d.blank_line_after_open_tag),
            control_space_parens: self.control_space_parens.unwrap_or(d.control_space_parens),
            closure_brace_style: self.closure_brace_style.unwrap_or(d.closure_brace_style),
            function_brace_style: self.function_brace_style.unwrap_or(d.function_brace_style),
            method_brace_style: self.method_brace_style.unwrap_or(d.method_brace_style),
            classlike_brace_style: self.classlike_brace_style.unwrap_or(d.classlike_brace_style),
            control_brace_style: self.control_brace_style.unwrap_or(d.control_brace_style),
            space_before_closure_params: self.space_before_closure_params.unwrap_or(d.space_before_closure_params),
            space_after_closure_use: self.space_after_closure_use.unwrap_or(d.space_after_closure_use),
            space_before_arrow_function_params: self
                .space_before_arrow_function_params
                .unwrap_or(d.space_before_arrow_function_params),
            static_before_visibility: self.static_before_visibility.unwrap_or(d.static_before_visibility),
            null_type_hint: self.null_type_hint.unwrap_or(d.null_type_hint),
            binary_op_spacing: self.binary_op_spacing.unwrap_or(d.binary_op_spacing),
            type_spacing: self.type_spacing.unwrap_or(d.type_spacing),
            method_chain_break_threshold: self.method_chain_break_threshold.unwrap_or(d.method_chain_break_threshold),
            break_promoted_properties_list: self
                .break_promoted_properties_list
                .unwrap_or(d.break_promoted_properties_list),
            space_concatenation: self.space_concatenation.unwrap_or(d.space_concatenation),

            preserve_multiline_binary_operations: self
                .preserve_multiline_binary_operations
                .unwrap_or(d.preserve_multiline_binary_operations),
            method_chain_breaking_style: self.method_chain_breaking_style.unwrap_or(d.method_chain_breaking_style),
        }
    }
}

impl ConfigurationEntry for FormatterConfiguration {
    fn configure<St: BuilderState>(self, builder: ConfigBuilder<St>) -> Result<ConfigBuilder<St>, Error> {
        Ok(builder)
    }
}
