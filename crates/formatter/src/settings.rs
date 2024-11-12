use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

/// Format settings for the PHP printer.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FormatSettings {
    /// Specify the maximum line length that the printer will wrap on.
    /// Default: 120
    pub print_width: usize,

    /// Specify the number of spaces per indentation-level.
    /// Default: 4
    pub tab_width: usize,

    /// Indent lines with tabs instead of spaces.
    pub use_tabs: bool,

    /// Specify which end-of-line characters to use.
    /// Default: "lf"
    pub end_of_line: EndOfLine,

    /// Use single quotes instead of double quotes for strings.
    /// Default: false
    pub single_quote: bool,

    /// Enable or disable trailing commas in multi-line syntactic structures.
    /// Default: true
    pub trailing_comma: bool,

    /// Add spaces around the `=` in declare statements.
    /// Default: false
    pub space_around_declare_equals: bool,

    /// Include a semicolon after `declare(strict_types=1)`.
    /// Default: true
    pub strict_types_semicolon: bool,

    /// Keyword casing (e.g., lowercase, uppercase).
    /// Default: lowercase
    pub keyword_case: CasingStyle,

    /// Casting operator for strings.
    /// Default: `(string)`
    pub string_cast: StringCastOperator,

    /// Casting operator for floats.
    /// Default: `(float)`
    pub float_cast: FloatCastOperator,

    /// Casting operator for booleans.
    /// Default: `(bool)`
    pub bool_cast: BoolCastOperator,

    /// Casting operator for integers.
    /// Default: `(int)`
    pub int_cast: IntCastOperator,

    /// Leave casting operators as is.
    /// Default: false
    pub leave_casts_as_is: bool,

    /// Include `?>` in files containing only PHP code.
    /// Default: false
    pub include_closing_tag: bool,

    /// Blank line after the opening PHP tag.
    /// Default: true
    pub blank_line_after_open_tag: bool,

    /// Controls whether a single breaking argument (e.g., an array or closure) is inlined within the enclosing parentheses.
    /// Default: true
    pub inline_single_breaking_argument: bool,

    /// Controls whether a single breaking attribute is inlined within the enclosing `#[` and `]`
    /// Default: true
    pub inline_single_attribute: bool,

    /// In a control structure expression, is there a space after the opening parenthesis
    ///  and a space before the closing parenthesis?
    ///
    /// Default: false
    pub control_space_parens: bool,

    /// Brace style for closures.
    pub closure_brace_style: BraceStyle,

    /// Brace style for function.
    pub function_brace_style: BraceStyle,

    /// Brace style for methods.
    pub method_brace_style: BraceStyle,

    /// Brace style for class-like structures.
    pub classlike_brace_style: BraceStyle,

    /// Brace style for control structures.
    pub control_brace_style: BraceStyle,

    /// Space between function name and opening parenthesis in calls.
    /// Default: false
    pub space_after_function_name: bool,

    /// Space between the `function` keyword and the opening parenthesis in closure declarations.
    /// Default: true
    pub space_before_closure_params: bool,

    /// Space between the `use` keyword and the opening parenthesis in closure use declarations.
    /// Default: true
    pub space_after_closure_use: bool,

    /// Space between the `fn` keyword and the opening parenthesis in arrow function declarations.
    /// Default: true
    pub space_before_arrow_function_params: bool,

    /// Order of `static` and visibility in method declarations.
    /// Default: Visibility first
    pub static_visibility_order: StaticVisibilityOrder,

    /// Require parentheses around class instantiations.
    /// Default: true
    pub require_instantiation_parens: bool,

    /// Sort methods alphabetically.
    /// Default: false
    pub sort_methods: bool,

    /// Sort properties alphabetically.
    /// Default: false
    pub sort_properties: bool,

    /// Sort enum cases alphabetically.
    /// Default: false
    pub sort_enum_cases: bool,

    /// Sort class-like constants alphabetically.
    /// Default: false
    pub sort_classlike_constants: bool,

    /// Ensure constructor is the first method.
    /// Default: false
    pub constructor_first: bool,

    /// Ensure destructor is the last method.
    /// Default: false
    pub destructor_last: bool,

    /// Static methods come before non-static methods.
    /// Default: false
    pub static_methods_first: bool,

    /// Static properties come before non-static properties.
    /// Default: false
    pub static_properties_first: bool,

    /// Split grouped `use` statements.
    /// Default: false
    pub split_use_statements: bool,

    /// Array style (`[a, b]` or `array(a, b)`).
    /// Default: Short
    pub array_style: ArrayStyle,

    /// List style (`[a, b]` or `list(a, b)`).
    /// Default: Short
    pub list_style: ListStyle,

    /// PHP attributes without arguments should (not) have empty parentheses.
    /// Default: WithParens
    pub attr_parens: OptionalParensStyle,

    /// Class instantiation with no arguments (`new Foo` or `new Foo()`).
    /// Default: WithParens
    pub empty_new_parens: OptionalParensStyle,

    /// Null type hint style (`null|foo` or `?foo`).
    /// Default: NullPipe
    pub null_type_hint: NullTypeHint,

    /// Spacing around binary operators.
    /// Default: 1
    pub binary_op_spacing: usize,

    /// Replace `<>` with `!=`.
    /// Default: true
    pub replace_angle_not_equals: bool,

    /// Spacing in union/intersection types (`A | B` or `A|B`).
    /// Default: 0
    pub type_spacing: usize,

    /// Split constants and properties into separate statements.
    /// Default: true
    pub split_multi_declare: bool,

    /// The minimum length of a method call chain that triggers line-breaking formatting.
    ///
    /// When the number of chained method calls exceeds this threshold, the formatter will break the chain into multiple lines:
    ///
    /// Default: 4
    pub method_chain_break_threshold: usize,

    /// Whether to break a parameter list into multiple lines if it contains one or more promoted property.
    ///
    /// Default: true
    pub break_promoted_properties_list: bool,

    /// Whether to add a space before and after the concatenation operator.
    ///
    /// Default: true
    pub space_concatenation: bool,

    /// Whether to preserve arrays that are already broken into multiple lines.
    ///
    /// If enabled, arrays that span multiple lines will remain in multiple lines,
    /// even if they can fit into a single line. This gives users the option to
    /// manually decide when an array should use a multi-line format for readability.
    ///
    /// Default: true
    pub preserve_broken_arrays: bool,

    /// Whether to preserve argument list that are already broken into multiple lines.
    ///
    /// If enabled, argum ent lists that span multiple lines will remain in multiple lines,
    /// even if they can fit into a single line. This gives users the option to
    /// manually decide when an argument list should use a multi-line format for readability.
    ///
    /// Default: true
    pub preserve_broken_argument_lists: bool,

    /// Whether to inline a single attribute group in a parameter.
    ///
    /// When enabled, a single attribute group applied to a parameter can be formatted
    /// inline with the parameter, instead of appearing on a separate line.
    ///
    /// Default: true
    pub inline_single_attribute_group: bool,

    /// Whether to preserve newlines between attribute groups.
    ///
    /// If an attribute group is already followed by a newline, this option can
    /// be used to preserve that newline.
    ///
    /// Default: true
    pub preserve_attribute_group_newlines: bool,

    /// Preserve existing newlines in parameter lists.
    ///
    /// If a parameter list is already broken into multiple lines, this option can
    /// be used to preserve the existing newlines.
    ///
    /// Default: true
    pub preserve_multiline_parameters: bool,

    /// Whether to preserve binary operations that are already broken into multiple lines.
    ///
    /// If enabled, binary operations that span multiple lines will remain in multiple lines,
    /// even if they can fit into a single line. This gives users the option to
    /// manually decide when a binary operation should use a multi-line format for readability.
    ///
    /// Default: true
    pub preserve_multiline_binary_operations: bool,

    /// How to format broken method/property chains.
    ///
    /// When breaking a method or property chain, this option determines whether the
    /// first method/property remains on the same line as the object/class, or if it starts on a new line.
    ///
    /// Default: SameLine
    pub method_chain_breaking_style: MethodChainBreakingStyle,
}

impl FormatSettings {
    pub fn fennec_style() -> Self {
        Self {
            print_width: 70,
            tab_width: 2,
            use_tabs: false,
            end_of_line: EndOfLine::default(),
            single_quote: true,
            trailing_comma: true,
            space_around_declare_equals: false,
            strict_types_semicolon: true,
            keyword_case: CasingStyle::Lowercase,
            string_cast: StringCastOperator::String,
            float_cast: FloatCastOperator::Float,
            bool_cast: BoolCastOperator::Bool,
            int_cast: IntCastOperator::Int,
            leave_casts_as_is: false,
            include_closing_tag: false,
            blank_line_after_open_tag: true,
            inline_single_breaking_argument: true,
            inline_single_attribute: true,
            control_space_parens: false,
            closure_brace_style: BraceStyle::SameLine,
            function_brace_style: BraceStyle::SameLine,
            method_brace_style: BraceStyle::SameLine,
            classlike_brace_style: BraceStyle::SameLine,
            control_brace_style: BraceStyle::SameLine,
            space_after_function_name: false,
            space_before_closure_params: false,
            space_after_closure_use: false,
            space_before_arrow_function_params: false,
            static_visibility_order: StaticVisibilityOrder::VisibilityFirst,
            require_instantiation_parens: true,
            sort_enum_cases: false,
            sort_classlike_constants: false,
            sort_methods: false,
            sort_properties: false,
            constructor_first: false,
            destructor_last: false,
            static_methods_first: false,
            static_properties_first: false,
            split_use_statements: true,
            array_style: ArrayStyle::Short,
            list_style: ListStyle::Short,
            attr_parens: OptionalParensStyle::WithoutParens,
            empty_new_parens: OptionalParensStyle::WithParens,
            null_type_hint: NullTypeHint::NullPipe,
            binary_op_spacing: 1,
            replace_angle_not_equals: true,
            type_spacing: 0,
            split_multi_declare: true,
            method_chain_break_threshold: 3,
            break_promoted_properties_list: true,
            space_concatenation: true,
            preserve_broken_arrays: false,
            preserve_broken_argument_lists: false,
            inline_single_attribute_group: false,
            preserve_attribute_group_newlines: true,
            preserve_multiline_parameters: false,
            preserve_multiline_binary_operations: false,
            method_chain_breaking_style: MethodChainBreakingStyle::NextLine,
        }
    }
}

impl Default for FormatSettings {
    /// Sets default values to align with best practices.
    fn default() -> Self {
        Self {
            print_width: 120,
            tab_width: 4,
            use_tabs: false,
            end_of_line: EndOfLine::default(),
            single_quote: false,
            trailing_comma: true,
            space_around_declare_equals: false,
            strict_types_semicolon: true,
            keyword_case: CasingStyle::default(),
            string_cast: StringCastOperator::default(),
            float_cast: FloatCastOperator::default(),
            bool_cast: BoolCastOperator::default(),
            int_cast: IntCastOperator::default(),
            leave_casts_as_is: false,
            include_closing_tag: false,
            blank_line_after_open_tag: true,
            inline_single_breaking_argument: true,
            inline_single_attribute: true,
            control_space_parens: false,
            closure_brace_style: BraceStyle::SameLine,
            function_brace_style: BraceStyle::NextLine,
            method_brace_style: BraceStyle::NextLine,
            classlike_brace_style: BraceStyle::NextLine,
            control_brace_style: BraceStyle::SameLine,
            space_after_function_name: false,
            space_before_closure_params: true,
            space_after_closure_use: true,
            space_before_arrow_function_params: false,
            static_visibility_order: StaticVisibilityOrder::default(),
            require_instantiation_parens: true,
            sort_enum_cases: false,
            sort_classlike_constants: false,
            sort_methods: false,
            sort_properties: false,
            constructor_first: false,
            destructor_last: false,
            static_methods_first: false,
            static_properties_first: false,
            split_use_statements: false,
            array_style: ArrayStyle::default(),
            list_style: ListStyle::default(),
            attr_parens: OptionalParensStyle::default(),
            empty_new_parens: OptionalParensStyle::default(),
            null_type_hint: NullTypeHint::default(),
            binary_op_spacing: 1,
            replace_angle_not_equals: true,
            type_spacing: 0,
            split_multi_declare: true,
            method_chain_break_threshold: 4,
            break_promoted_properties_list: true,
            space_concatenation: true,
            preserve_broken_arrays: true,
            preserve_broken_argument_lists: true,
            inline_single_attribute_group: true,
            preserve_attribute_group_newlines: true,
            preserve_multiline_parameters: true,
            preserve_multiline_binary_operations: true,
            method_chain_breaking_style: MethodChainBreakingStyle::SameLine,
        }
    }
}

/// Specifies the style of line endings.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum EndOfLine {
    #[default]
    #[serde(alias = "auto")]
    Auto,
    #[serde(alias = "lf")]
    Lf,
    #[serde(alias = "crlf")]
    Crlf,
    #[serde(alias = "cr")]
    Cr,
}

/// Specifies the style of line endings.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CasingStyle {
    #[default]
    #[serde(alias = "lowercase", alias = "lower")]
    Lowercase,
    #[serde(alias = "uppercase", alias = "upper")]
    Uppercase,
}

/// Specifies the style of line endings.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum BraceStyle {
    #[serde(alias = "same")]
    SameLine,
    #[serde(alias = "next")]
    NextLine,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum MethodChainBreakingStyle {
    #[serde(alias = "same")]
    #[default]
    SameLine,
    #[serde(alias = "next")]
    NextLine,
}

impl BraceStyle {
    #[inline]
    pub fn is_next_line(&self) -> bool {
        *self == Self::NextLine
    }
}

impl MethodChainBreakingStyle {
    #[inline]
    pub fn is_next_line(&self) -> bool {
        *self == Self::NextLine
    }
}

impl EndOfLine {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Crlf => "\r\n",
            Self::Cr => "\r",
            Self::Lf | Self::Auto => "\n",
        }
    }
}

impl FromStr for EndOfLine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "crlf" => Self::Crlf,
            "cr" => Self::Cr,
            "auto" => Self::Auto,
            "lf" => Self::Lf,
            _ => Self::default(),
        })
    }
}

/// Specifies the order of `static` and visibility in method declarations.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum StaticVisibilityOrder {
    #[default]
    #[serde(alias = "visibility")]
    VisibilityFirst,
    #[serde(alias = "static")]
    StaticFirst,
}

/// Casting operator for strings.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum StringCastOperator {
    #[default]
    #[serde(alias = "(string)", alias = "string")]
    String,
    #[serde(alias = "(bianry)", alias = "binary")]
    Binary,
}

/// Casting operator for floats.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum FloatCastOperator {
    #[default]
    #[serde(alias = "(float)", alias = "float")]
    Float,
    #[serde(alias = "(double)", alias = "double")]
    Double,
    #[serde(alias = "(real)", alias = "real")]
    Real,
}

/// Casting operator for booleans.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum BoolCastOperator {
    #[default]
    #[serde(alias = "(bool)", alias = "bool")]
    Bool,
    #[serde(alias = "(boolean)", alias = "boolean")]
    Boolean,
}

/// Casting operator for integers.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum IntCastOperator {
    #[default]
    #[serde(alias = "(int)", alias = "int")]
    Int,
    #[serde(alias = "(integer)", alias = "integer")]
    Integer,
}

/// Specifies array style.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ArrayStyle {
    #[default]
    #[serde(alias = "short", alias = "[]")]
    Short,
    #[serde(alias = "long", alias = "legacy", alias = "array()")]
    Long,
}

/// Specifies list style.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ListStyle {
    #[default]
    #[serde(alias = "short", alias = "[]")]
    Short,
    #[serde(alias = "long", alias = "legacy", alias = "list()")]
    Long,
}

/// Specifies optional parentheses style.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum OptionalParensStyle {
    #[default]
    #[serde(alias = "with_parens", alias = "with")]
    WithParens,
    #[serde(alias = "without_parens", alias = "without")]
    WithoutParens,
}

/// Specifies null type hint style.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum NullTypeHint {
    #[default]
    #[serde(alias = "null_pipe", alias = "pipe", alias = "long", alias = "|")]
    NullPipe,
    #[serde(alias = "question", alias = "short", alias = "?")]
    Question,
}

impl NullTypeHint {
    pub fn is_question(&self) -> bool {
        *self == Self::Question
    }
}
