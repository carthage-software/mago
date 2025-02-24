use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

/// Format settings for the PHP printer.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FormatSettings {
    /// Specify the maximum line length that the printer will wrap on.
    ///
    /// Default: 120
    #[serde(default = "default_print_width")]
    pub print_width: usize,

    /// Specify the number of spaces per indentation-level.
    ///
    /// Default: 4
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,

    /// Indent lines with tabs instead of spaces.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub use_tabs: bool,

    /// Specify which end-of-line characters to use.
    ///
    /// Default: "lf"
    #[serde(default)]
    pub end_of_line: EndOfLine,

    /// Use single quotes instead of double quotes for strings.
    ///
    /// The formatter will automatically determine whether to use single or double quotes based on the content of the string,
    /// with a preference for single quotes if this option is enabled.
    ///
    /// If the string contains more single quotes than double quotes, the formatter will use double quotes.
    /// If the string contains more double quotes than single quotes, the formatter will use single quotes.
    ///
    /// If the string contains an equal number of single and double quotes, the formatter will use single quotes
    /// if this option is enabled, and double quotes otherwise.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub single_quote: bool,

    /// Enable or disable trailing commas in multi-line syntactic structures.
    ///
    /// When enabled, the formatter will add a trailing comma to the last element in a multi-line list, array,
    /// parameter list, argument list, and other syntactic structures.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub trailing_comma: bool,

    /// Add spaces around the `=` in declare statements.
    ///
    /// When enabled, the formatter will add a space before and after the `=` in declare statements.
    ///
    /// Example:
    ///
    /// ```php
    /// declare(strict_types = 1);
    /// ```
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_around_declare_equals: bool,

    /// Keyword casing (e.g., lowercase, uppercase).
    ///
    /// The formatter will convert keywords to the specified case.
    ///
    /// Example:
    ///
    /// ```php
    /// // lowercase
    /// if (true) {
    ///    $foo = (string) $bar;
    ///
    ///    echo $foo;
    ///
    ///    return;
    /// }
    ///
    /// // uppercase
    /// IF (TRUE) {
    ///   $foo = (STRING) $bar;
    ///
    ///   ECHO $foo;
    ///
    ///   RETURN;
    /// }
    /// ```
    ///
    /// Default: lowercase
    #[serde(default)]
    pub keyword_case: CasingStyle,

    /// Blank line after the opening PHP tag.
    ///
    /// When enabled, the formatter will add a blank line after the opening PHP tag.
    ///
    /// Example:
    ///
    /// ```php
    /// <?php
    ///
    /// echo 'Hello, world!';
    /// ```
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub blank_line_after_open_tag: bool,

    /// In a control structure expression, is there a space after the opening parenthesis
    ///  and a space before the closing parenthesis?
    ///
    /// When enabled, the formatter will add a space after the opening parenthesis and a space before the closing parenthesis
    /// in control structure expressions.
    ///
    /// Example:
    ///
    /// ```php
    /// if ( $expr ) {
    /// }
    ///
    /// // or
    ///
    /// if ($expr) {
    /// }
    /// ```
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub control_space_parens: bool,

    /// Whether the formatter should keep the opening brace on the same line for closures, or move it to the next line.
    ///
    /// Example:
    ///
    /// ```php
    /// $closure = function() {
    ///    return 'Hello, world!';
    /// };
    ///
    /// // or
    ///
    /// $closure = function()
    /// {
    ///   return 'Hello, world!';
    /// };
    /// ```
    ///
    ///
    /// Default: same_line
    #[serde(default = "BraceStyle::same_line")]
    pub closure_brace_style: BraceStyle,

    /// Whether the formatter should keep the opening brace on the same line for functions, or move it to the next line.
    ///
    /// Example:
    ///
    /// ```php
    /// function foo() {
    ///   return 'Hello, world!';
    /// }
    ///
    /// // or
    ///
    /// function foo()
    /// {
    ///   return 'Hello, world!';
    /// }
    /// ```
    ///
    /// Default: next_line
    #[serde(default = "BraceStyle::next_line")]
    pub function_brace_style: BraceStyle,

    /// Whether the formatter should keep the opening brace on the same line for methods, or move it to the next line.
    ///
    /// Example:
    ///
    /// ```php
    /// class Foo
    /// {
    ///   public function bar() {
    ///     return 'Hello, world!';
    ///   }
    ///
    ///   // or
    ///
    ///   public function bar()
    ///   {
    ///     return 'Hello, world!';
    ///   }
    /// }
    /// ```
    ///
    /// Default: next_line
    #[serde(default = "BraceStyle::next_line")]
    pub method_brace_style: BraceStyle,

    /// Whether the formatter should keep the opening brace on the same line for class-like structures, or move it to the next line.
    ///
    /// Example:
    ///
    /// ```php
    /// class Foo {
    /// }
    ///
    /// interface Bar {
    /// }
    ///
    /// trait Baz {
    /// }
    ///
    /// enum Qux {
    /// }
    ///
    /// // or
    ///
    /// class Foo
    /// {
    /// }
    ///
    /// interface Bar
    /// {
    /// }
    ///
    /// trait Baz
    /// {
    /// }
    ///
    /// enum Qux
    /// {
    /// }
    /// ```
    ///
    /// Default: next_line
    #[serde(default = "BraceStyle::next_line")]
    pub classlike_brace_style: BraceStyle,

    /// Whether the formatter should keep the opening brace of a block statement on the same line for control structures,
    /// or move it to the next line.
    ///
    /// Example:
    ///
    /// ```php
    /// if ($expr) {
    ///   return 'Hello, world!';
    /// }
    ///
    /// // or
    ///
    /// if ($expr)
    /// {
    ///   return 'Hello, world!';
    /// }
    /// ```
    ///
    /// Default: same_line
    #[serde(default = "BraceStyle::same_line")]
    pub control_brace_style: BraceStyle,

    /// Whether to add a space between the `function` keyword and the opening parenthesis in closure declarations,
    /// or keep them together.
    ///
    /// Example:
    ///
    /// ```php
    /// $closure = function () {
    ///   return 'Hello, world!';
    /// };
    ///
    /// // or
    ///
    /// $closure = function() {
    ///   return 'Hello, world!';
    /// };
    /// ```
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_closure_params: bool,

    /// Whether to add a space between the `use` keyword and the opening parenthesis in closure use declarations.
    ///
    /// Example:
    ///
    /// ```php
    /// $closure = function() use ($foo, $bar) {
    ///   return 'Hello, world!';
    /// };
    ///
    /// // or
    ///
    /// $closure = function() use($foo, $bar) {
    ///   return 'Hello, world!';
    /// };
    /// ```
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_after_closure_use: bool,

    /// Whether to add a space between the `fn` keyword and the opening parenthesis in arrow function declarations.
    ///
    /// Example:
    ///
    /// ```php
    /// $closure = fn () => 'Hello, world!';
    ///
    /// // or
    ///
    /// $closure = fn() => 'Hello, world!';
    /// ```
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_arrow_function_params: bool,

    /// Whether to put the `static` keyword before the visibility keyword.
    ///
    /// Example:
    ///
    /// ```php
    /// class Foo {
    ///   public static $bar;
    ///
    ///   // or
    ///
    ///   static public $bar;
    /// }
    /// ```
    ///
    /// This setting also affects the order of the `readonly` keyword, if present.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub static_before_visibility: bool,

    /// Which style to use for null type hints.
    ///
    /// Example:
    ///
    /// ```php
    /// function foo(?string $bar) {
    ///   return $bar;
    /// }
    ///
    /// // or
    ///
    /// function foo(null|string $bar) {
    ///   return $bar;
    /// }
    /// ```
    ///
    /// Default: NullPipe
    #[serde(default)]
    pub null_type_hint: NullTypeHint,

    /// How many spaces to add around type operators.
    ///
    /// Example:
    ///
    /// ```php
    /// function foo(): A|B {}
    /// function bar(): A&(B|C) {}
    /// function baz(): ?B {}
    ///
    /// // or
    ///
    /// function foo(): A | B {}
    /// function bar(): A & ( B | C) {}
    /// function baz(): ? B {}
    /// ```
    ///
    /// Default: 0
    #[serde(default = "default_type_spacing")]
    pub type_spacing: usize,

    /// The minimum length of a method call chain that triggers line-breaking formatting.
    ///
    /// When the number of chained method calls exceeds this threshold, the formatter will break the chain into multiple lines.
    ///
    /// Default: 4
    #[serde(default = "default_method_chain_break_threshold")]
    pub method_chain_break_threshold: usize,

    /// Whether to break a parameter list into multiple lines if it contains one or more promoted property even if it fits into a single line.
    ///
    /// Example:
    ///
    /// ```php
    /// class User {
    ///   public function __construct(
    ///     public string $name,
    ///     public string $email,
    ///   ) {}
    /// }
    ///
    /// // or
    ///
    /// class User {
    ///   public function __construct(public string $name, public string $email) {}
    /// }
    /// ```
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub break_promoted_properties_list: bool,

    /// Whether to add a space before and after the concatenation operator.
    ///
    /// Example:
    ///
    /// ```php
    /// $foo = 'Hello, ' . 'world!';
    ///
    /// // or
    ///
    /// $foo = 'Hello, '.'world!';
    /// ```
    ///
    /// Note: The number of spaces added around the concatenation operator is controlled by the `binary_op_spacing` setting.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_concatenation: bool,

    /// How to format broken method/property chains.
    ///
    /// When breaking a method or property chain, this option determines whether the
    /// first method/property remains on the same line as the object/class, or if it starts on a new line.
    ///
    /// Example:
    ///
    /// ```php
    /// $foo->bar()
    ///   ->baz();
    ///
    /// // or
    ///
    /// $foo
    ///   ->bar()
    ///   ->baz();
    /// ```
    ///
    /// Default: NextLine
    #[serde(default)]
    pub method_chain_breaking_style: MethodChainBreakingStyle,

    /// Whether to add a line before binary operators or after when breaking.
    ///
    /// Note: This setting will always be false if the rhs of the binary operator has a leading comment.
    ///
    /// Example:
    ///
    /// ```php
    /// // line_before_binary_operator = true
    /// $foo = 'Hello, '
    ///     . 'world!';
    ///
    /// // line_before_binary_operator = true
    /// $foo = 'Hello, ' .
    ///     /**
    ///      * Comment
    ///      */
    ///     'world!';
    ///
    /// // line_before_binary_operator = false
    /// $foo = 'Hello, ' .
    ///     'world!';
    ///
    /// // line_before_binary_operator = false
    /// $foo = 'Hello, ' .
    ///     /**
    ///      * Comment
    ///      */
    ///     'world!';
    /// ```
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub line_before_binary_operator: bool,

    /// Whether to sort use statements alphabetically.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub sort_uses: bool,

    /// Whether to insert a blank line between different types of use statements
    /// (e.g., classes, functions, constants).
    ///
    /// Default: true
    ///
    /// Example:
    ///
    /// ```php
    /// use Foo\Bar;
    /// use Foo\Baz;
    ///
    /// use function Foo\bar;
    /// use function Foo\baz;
    ///
    /// use const Foo\A;
    /// use const Foo\B;
    ///
    /// // or, if separate_use_types is false:
    ///
    /// use Foo\Bar;
    /// use Foo\Baz;
    /// use function Foo\bar;
    /// use function Foo\baz;
    /// use const Foo\A;
    /// use const Foo\B;
    /// ```
    #[serde(default = "default_true")]
    pub separate_use_types: bool,

    /// Whether to expand grouped use statements into individual statements.
    ///
    /// Default: true
    ///
    /// Example:
    ///
    /// ```php
    /// use Foo\{Bar, Baz};
    ///
    /// // or, if expand_use_groups is true:
    ///
    /// use Foo\Bar;
    /// use Foo\Baz;
    /// ```
    #[serde(default = "default_true")]
    pub expand_use_groups: bool,

    /// Whether to remove the trailing close tag.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub remove_trailing_close_tag: bool,
}

impl Default for FormatSettings {
    /// Sets default values to align with best practices.
    fn default() -> Self {
        Self {
            print_width: default_print_width(),
            tab_width: default_tab_width(),
            use_tabs: false,
            end_of_line: EndOfLine::default(),
            single_quote: true,
            trailing_comma: true,
            space_around_declare_equals: false,
            keyword_case: CasingStyle::default(),
            blank_line_after_open_tag: true,
            control_space_parens: false,
            closure_brace_style: BraceStyle::SameLine,
            function_brace_style: BraceStyle::NextLine,
            method_brace_style: BraceStyle::NextLine,
            classlike_brace_style: BraceStyle::NextLine,
            control_brace_style: BraceStyle::SameLine,
            space_before_closure_params: true,
            space_after_closure_use: true,
            space_before_arrow_function_params: false,
            static_before_visibility: false,
            null_type_hint: NullTypeHint::default(),
            type_spacing: default_type_spacing(),
            method_chain_break_threshold: default_method_chain_break_threshold(),
            break_promoted_properties_list: true,
            space_concatenation: true,
            method_chain_breaking_style: MethodChainBreakingStyle::NextLine,
            line_before_binary_operator: false,
            sort_uses: true,
            separate_use_types: true,
            expand_use_groups: true,
            remove_trailing_close_tag: true,
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
    #[serde(alias = "same_line")]
    SameLine,
    #[serde(alias = "next_line")]
    NextLine,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum MethodChainBreakingStyle {
    #[serde(alias = "same_line")]
    SameLine,
    #[default]
    #[serde(alias = "next_line")]
    NextLine,
}

impl BraceStyle {
    pub fn same_line() -> Self {
        Self::SameLine
    }

    pub fn next_line() -> Self {
        Self::NextLine
    }

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
    #[inline(always)]
    pub const fn as_str(&self) -> &'static str {
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

fn default_print_width() -> usize {
    120
}

fn default_tab_width() -> usize {
    4
}

fn default_type_spacing() -> usize {
    0
}

fn default_method_chain_break_threshold() -> usize {
    4
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}
