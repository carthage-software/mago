use std::str::FromStr;

use crate::presets::FormatterPreset;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Macro to generate both `FormatSettings` and `RawFormatSettings` from a single definition.
///
/// This ensures the two structs stay in sync. `RawFormatSettings` has all fields as `Option<T>`
/// to track which fields were explicitly set by the user during deserialization.
///
/// All fields MUST have an explicit default function specified using `=> "default_fn"` syntax.
/// For types that implement Default, use the type's default method (e.g., `=> "EndOfLine::default"`).
macro_rules! generate_formatter_settings {
    (
        $(
            $(#[$field_meta:meta])*
            $field:ident : $type:ty => $default:literal
        ),* $(,)?
    ) => {
        /// Format settings for the PHP printer.
        ///
        /// **WARNING:** This structure is not to be considered exhaustive. New fields may be added in minor
        /// or patch releases. Do not construct this structure directly outside of the formatter crate.
        ///
        /// New fields are added with default values to ensure backward compatibility,
        /// unless a breaking change is explicitly intended for PER-CS compliance updates.
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, JsonSchema)]
        #[serde(rename_all = "kebab-case", deny_unknown_fields)]
        pub struct FormatSettings {
            $(
                $(#[$field_meta])*
                #[serde(default = $default)]
                pub $field: $type,
            )*
        }

        /// Raw format settings with all fields optional.
        ///
        /// Used during deserialization to track which fields were explicitly set by the user.
        /// This allows us to distinguish between "user set a value" and "serde applied a default".
        #[derive(Debug, Clone, Default, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub struct RawFormatSettings {
            $(
                $(#[$field_meta])*
                pub $field: Option<$type>,
            )*
        }

        impl RawFormatSettings {
            /// Merge raw user settings over a base (preset or default).
            ///
            /// If user explicitly set a field (Some), use their value.
            /// Otherwise, use the base value.
            pub fn merge_with(self, base: FormatSettings) -> FormatSettings {
                FormatSettings {
                    $(
                        $field: self.$field.unwrap_or(base.$field),
                    )*
                }
            }
        }
    };
}

generate_formatter_settings! {
    /// Maximum line length that the printer will wrap on.
    ///
    /// Default: 120
    print_width: usize => "default_print_width",

    /// Number of spaces per indentation level.
    ///
    /// Default: 4
    tab_width: usize => "default_tab_width",

    /// Whether to use tabs instead of spaces for indentation.
    ///
    /// Default: false
    use_tabs: bool => "default_false",

    /// End-of-line characters to use.
    ///
    /// Default: "lf"
    end_of_line: EndOfLine => "EndOfLine::default",

    /// Whether to use single quotes instead of double quotes for strings.
    ///
    /// The formatter automatically determines which quotes to use based on the string content,
    /// with a preference for single quotes if this option is enabled.
    ///
    /// Decision logic:
    /// - If the string contains more single quotes than double quotes, double quotes are used
    /// - If the string contains more double quotes than single quotes, single quotes are used
    /// - If equal number of both, single quotes are used if this option is true
    ///
    /// Default: true
    single_quote: bool => "default_true",

    /// Whether to add a trailing comma to the last element in multi-line syntactic structures.
    ///
    /// When enabled, trailing commas are added to lists, arrays, parameter lists,
    /// argument lists, and other similar structures when they span multiple lines.
    ///
    /// Default: true
    trailing_comma: bool => "default_true",

    /// Whether to remove the trailing PHP close tag (`?>`) from files.
    ///
    /// Default: true
    remove_trailing_close_tag: bool => "default_true",

    /// Brace placement for control structures (if, for, while, etc.).
    ///
    /// Example with `same_line`:
    /// ```php
    /// if ($expr) {
    ///     return 'Hello, world!';
    /// }
    /// ```
    ///
    /// Example with `next_line`:
    /// ```php
    /// if ($expr)
    /// {
    ///     return 'Hello, world!';
    /// }
    /// ```
    ///
    /// Default: `same_line`
    control_brace_style: BraceStyle => "BraceStyle::same_line",

    /// Whether to place `else`, `elseif`, `catch` and `finally` on a new line.
    ///
    /// Default: false
    following_clause_on_newline: bool => "default_false",

    /// Brace placement for closures.
    ///
    /// Example with `same_line`:
    /// ```php
    /// $closure = function() {
    ///     return 'Hello, world!';
    /// };
    /// ```
    ///
    /// Example with `next_line`:
    /// ```php
    /// $closure = function()
    /// {
    ///     return 'Hello, world!';
    /// };
    /// ```
    ///
    /// Default: `same_line`
    closure_brace_style: BraceStyle => "BraceStyle::same_line",

    /// Brace placement for function declarations.
    ///
    /// Example with `same_line`:
    /// ```php
    /// function foo() {
    ///     return 'Hello, world!';
    /// }
    /// ```
    ///
    /// Example with `next_line`:
    /// ```php
    /// function foo()
    /// {
    ///     return 'Hello, world!';
    /// }
    /// ```
    ///
    /// Default: `next_line`
    function_brace_style: BraceStyle => "BraceStyle::next_line",

    /// Brace placement for method declarations.
    ///
    /// Example with `same_line`:
    /// ```php
    /// class Foo
    /// {
    ///     public function bar() {
    ///         return 'Hello, world!';
    ///     }
    /// }
    /// ```
    ///
    /// Example with `next_line`:
    /// ```php
    /// class Foo
    /// {
    ///     public function bar()
    ///     {
    ///         return 'Hello, world!';
    ///     }
    /// }
    /// ```
    ///
    /// Default: `next_line`
    method_brace_style: BraceStyle => "BraceStyle::next_line",

    /// Brace placement for class-like structures (classes, interfaces, traits, enums).
    ///
    /// Example with `same_line`:
    /// ```php
    /// class Foo {
    /// }
    /// ```
    ///
    /// Example with `next_line` or `always_next_line`:
    /// ```php
    /// class Foo
    /// {
    /// }
    /// ```
    ///
    /// Default: `always_next_line`
    classlike_brace_style: BraceStyle => "BraceStyle::always_next_line",

    /// Place empty control structure bodies on the same line.
    ///
    /// Example with `false`:
    /// ```php
    /// if ($expr)
    /// {
    /// }
    /// ```
    ///
    /// Example with `true`:
    /// ```php
    /// if ($expr) {}
    /// ```
    ///
    /// Default: false
    inline_empty_control_braces: bool => "default_false",

    /// Place empty closure bodies on the same line.
    ///
    /// Example with `false`:
    /// ```php
    /// $closure = function()
    /// {
    /// };
    /// ```
    ///
    /// Example with `true`:
    /// ```php
    /// $closure = function() {};
    /// ```
    ///
    /// Default: true
    inline_empty_closure_braces: bool => "default_true",

    /// Place empty function bodies on the same line.
    ///
    /// Example with `false`:
    /// ```php
    /// function foo()
    /// {
    /// }
    /// ```
    ///
    /// Example with `true`:
    /// ```php
    /// function foo() {}
    /// ```
    ///
    /// Default: true
    inline_empty_function_braces: bool => "default_true",

    /// Place empty method bodies on the same line.
    ///
    /// Example with `false`:
    /// ```php
    /// class Foo
    /// {
    ///     public function bar()
    ///     {
    ///     }
    /// }
    /// ```
    ///
    /// Example with `true`:
    /// ```php
    /// class Foo
    /// {
    ///     public function bar() {}
    /// }
    /// ```
    ///
    /// Default: true
    inline_empty_method_braces: bool => "default_true",

    /// Place empty constructor bodies on the same line.
    ///
    /// Example with `false`:
    /// ```php
    /// class Foo {
    ///     public function __construct()
    ///     {
    ///     }
    /// }
    /// ```
    ///
    /// Example with `true`:
    /// ```php
    /// class Foo {
    ///     public function __construct() {}
    /// }
    /// ```
    ///
    /// Default: true
    inline_empty_constructor_braces: bool => "default_true",

    /// Place empty class-like bodies on the same line.
    ///
    /// Example with `false`:
    /// ```php
    /// class Foo
    /// {
    /// }
    /// ```
    ///
    /// Example with `true`:
    /// ```php
    /// class Foo {}
    /// ```
    ///
    /// Default: true
    inline_empty_classlike_braces: bool => "default_true",

    /// Place empty anonymous class bodies on the same line.
    ///
    /// Example with `false`:
    /// ```php
    /// $anon = new class
    /// {
    /// };
    /// ```
    ///
    /// Example with `true`:
    /// ```php
    /// $anon = new class {};
    /// ```
    ///
    /// Default: true
    inline_empty_anonymous_class_braces: bool => "default_true",

    /// How to format broken method/property chains.
    ///
    /// When `next_line`, the first method/property starts on a new line:
    /// ```php
    /// $foo
    ///     ->bar()
    ///     ->baz();
    /// ```
    ///
    /// When `same_line`, the first method/property stays on the same line:
    /// ```php
    /// $foo->bar()
    ///     ->baz();
    /// ```
    ///
    /// Default: `next_line`
    method_chain_breaking_style: MethodChainBreakingStyle => "MethodChainBreakingStyle::default",

    /// When method chaining breaks across lines, place the first method on a new line.
    ///
    /// This follows PER-CS 4.7: "When [method chaining is] put on separate lines, [...] the first method MUST be on the next line."
    ///
    /// When enabled:
    /// ```php
    /// $this
    ///     ->getCache()
    ///     ->forget();
    /// ```
    ///
    /// When disabled:
    /// ```php
    /// $this->getCache()
    ///     ->forget();
    /// ```
    ///
    /// Default: `true`
    first_method_chain_on_new_line: bool => "default_true",

    /// Whether to preserve line breaks in method chains, even if they could fit on a single line.
    ///
    /// Default: false
    preserve_breaking_member_access_chain: bool => "default_false",

    /// Whether to preserve line breaks in argument lists, even if they could fit on a single line.
    ///
    /// Default: false
    preserve_breaking_argument_list: bool => "default_false",

    /// Whether to preserve line breaks in array-like structures, even if they could fit on a single line.
    ///
    /// Default: true
    preserve_breaking_array_like: bool => "default_true",

    /// Whether to preserve line breaks in parameter lists, even if they could fit on a single line.
    ///
    /// Default: false
    preserve_breaking_parameter_list: bool => "default_false",

    /// Whether to preserve line breaks in attribute lists, even if they could fit on a single line.
    ///
    /// Default: false
    preserve_breaking_attribute_list: bool => "default_false",

    /// Whether to preserve line breaks in conditional (ternary) expressions.
    ///
    /// Default: false
    preserve_breaking_conditional_expression: bool => "default_false",

    /// Whether to break a parameter list with one or more promoted properties into multiple lines.
    ///
    /// When enabled, parameter lists with promoted properties are always multi-line:
    /// ```php
    /// class User {
    ///     public function __construct(
    ///         public string $name,
    ///         public string $email,
    ///     ) {}
    /// }
    /// ```
    ///
    /// When disabled, they may be kept on a single line if space allows:
    /// ```php
    /// class User {
    ///     public function __construct(public string $name, public string $email) {}
    /// }
    /// ```
    ///
    /// Default: true
    break_promoted_properties_list: bool => "default_true",

    /// Whether to add a line before binary operators or after when breaking.
    ///
    /// When true:
    /// ```php
    /// $foo = 'Hello, '
    ///     . 'world!';
    /// ```
    ///
    /// When false:
    /// ```php
    /// $foo = 'Hello, ' .
    ///     'world!';
    /// ```
    ///
    /// Note: If the right side has a leading comment, this setting is always false.
    ///
    /// Default: true
    line_before_binary_operator: bool => "default_true",

    /// Whether to always break named argument lists into multiple lines.
    ///
    /// When enabled:
    /// ```php
    /// $foo = some_function(
    ///     argument1: 'value1',
    ///     argument2: 'value2',
    /// );
    /// ```
    ///
    /// Default: false
    always_break_named_arguments_list: bool => "default_false",

    /// Whether to always break named argument lists in attributes into multiple lines.
    ///
    /// When enabled:
    /// ```php
    /// #[SomeAttribute(
    ///     argument1: 'value1',
    ///     argument2: 'value2',
    /// )]
    /// class Foo {}
    /// ```
    ///
    /// Default: false
    always_break_attribute_named_argument_lists: bool => "default_false",

    /// Whether to use table-style alignment for arrays.
    ///
    /// When enabled, array elements are aligned in a table-like format:
    /// ```php
    /// $array = [
    ///     ['foo',  1.2,  123, false],
    ///     ['bar',  52.4, 456, true],
    ///     ['baz',  3.6,  789, false],
    ///     ['qux',  4.8,    1, true],
    ///     ['quux', 5.0,   12, false],
    /// ];
    /// ```
    ///
    /// Default: true
    array_table_style_alignment: bool => "default_true",

    /// Whether to align consecutive assignment-like constructs in columns.
    ///
    /// When enabled, consecutive variable assignments, class properties, class constants,
    /// global constants, array key-value pairs, and backed enum cases are column-aligned.
    ///
    /// Example with `true`:
    /// ```php
    /// $foo     = 1;
    /// $b       = 2;
    /// $ccccccc = 3;
    ///
    /// class X {
    ///     public string       $foo    = 1;
    ///     public readonly int $barrrr = 2;
    /// }
    /// ```
    ///
    /// Example with `false`:
    /// ```php
    /// $foo = 1;
    /// $b = 2;
    /// $ccccccc = 3;
    /// ```
    ///
    /// Note: Blank lines and comments break alignment runs. In class bodies,
    /// different member types (properties vs constants) are aligned separately.
    ///
    /// Default: false
    align_assignment_like: bool => "default_false",

    /// Whether to sort use statements alphabetically.
    ///
    /// Default: true
    sort_uses: bool => "default_true",

    /// Whether to sort class methods by visibility and name.
    ///
    /// When enabled, methods in class-like structures are automatically reordered:
    /// 1. Constructor (`__construct`) - always first
    /// 2. Static methods (by visibility: public, protected, private)
    ///    - Abstract methods before concrete methods
    ///    - Alphabetically by name within each group
    /// 3. Instance methods (by visibility: public, protected, private)
    ///    - Abstract methods before concrete methods
    ///    - Alphabetically by name within each group
    /// 4. Other magic methods (e.g., `__toString`, `__get`, `__set`)
    ///    - Sorted alphabetically by name
    /// 5. Destructor (`__destruct`) - always last
    ///
    /// This applies to all class-like structures: classes, traits, interfaces, and enums.
    /// Other members (constants, properties, trait uses, enum cases) remain in their original positions.
    ///
    /// Default: false
    sort_class_methods: bool => "default_false",

    /// Whether to insert a blank line between different types of use statements.
    ///
    /// When enabled:
    /// ```php
    /// use Foo\Bar;
    /// use Foo\Baz;
    ///
    /// use function Foo\bar;
    /// use function Foo\baz;
    ///
    /// use const Foo\A;
    /// use const Foo\B;
    /// ```
    ///
    /// When disabled:
    /// ```php
    /// use Foo\Bar;
    /// use Foo\Baz;
    /// use function Foo\bar;
    /// use function Foo\baz;
    /// use const Foo\A;
    /// use const Foo\B;
    /// ```
    ///
    /// Default: true
    separate_use_types: bool => "default_true",

    /// Whether to expand grouped use statements into individual statements.
    ///
    /// When enabled:
    /// ```php
    /// use Foo\Bar;
    /// use Foo\Baz;
    /// ```
    ///
    /// When disabled:
    /// ```php
    /// use Foo\{Bar, Baz};
    /// ```
    ///
    /// Default: true
    expand_use_groups: bool => "default_true",

    /// How to format null type hints.
    ///
    /// With `Question`:
    /// ```php
    /// function foo(?string $bar) {
    ///     return $bar;
    /// }
    /// ```
    ///
    /// With `NullPipe`:
    /// ```php
    /// function foo(null|string $bar) {
    ///     return $bar;
    /// }
    /// ```
    ///
    /// Default: `Question`
    null_type_hint: NullTypeHint => "NullTypeHint::default",

    /// Whether to include parentheses around `new` when followed by a member access.
    ///
    /// Controls whether to use PHP 8.4's shorthand syntax for new expressions
    /// followed by member access. If PHP version is earlier than 8.4, this is always true.
    ///
    /// When enabled:
    /// ```php
    /// $foo = (new Foo)->bar();
    /// ```
    ///
    /// When disabled (PHP 8.4+ only):
    /// ```php
    /// $foo = new Foo->bar();
    /// ```
    ///
    /// Default: false
    parentheses_around_new_in_member_access: bool => "default_false",

    /// Whether to include parentheses in `new` expressions when no arguments are provided.
    ///
    /// When enabled:
    /// ```php
    /// $foo = new Foo();
    /// ```
    ///
    /// When disabled:
    /// ```php
    /// $foo = new Foo;
    /// ```
    ///
    /// Default: true
    parentheses_in_new_expression: bool => "default_true",

    /// Whether to include parentheses in `exit` and `die` constructs.
    ///
    /// When enabled:
    /// ```php
    /// exit();
    /// die();
    /// ```
    ///
    /// When disabled:
    /// ```php
    /// exit;
    /// die;
    /// ```
    ///
    /// Default: true
    parentheses_in_exit_and_die: bool => "default_true",

    /// Whether to include parentheses in attributes with no arguments.
    ///
    /// When enabled:
    /// ```php
    /// #[SomeAttribute()]
    /// class Foo {}
    /// ```
    ///
    /// When disabled:
    /// ```php
    /// #[SomeAttribute]
    /// class Foo {}
    /// ```
    ///
    /// Default: false
    parentheses_in_attribute: bool => "default_false",

    /// Whether to add a space before the opening parameters in arrow functions.
    ///
    /// When enabled: `fn ($x) => $x * 2`
    /// When disabled: `fn($x) => $x * 2`
    ///
    /// Default: false
    space_before_arrow_function_parameter_list_parenthesis: bool => "default_false",

    /// Whether to add a space before the opening parameters in closures.
    ///
    /// When enabled: `function ($x) use ($y)`
    /// When disabled: `function($x) use ($y)`
    ///
    /// Default: true
    space_before_closure_parameter_list_parenthesis: bool => "default_true",

    /// Whether to add a space before the opening parameters in hooks.
    ///
    /// When enabled: `$hook ($param)`
    /// When disabled: `$hook($param)`
    ///
    /// Default: false
    space_before_hook_parameter_list_parenthesis: bool => "default_false",

    /// Whether to keep abstract property hooks inline.
    ///
    /// When enabled: `public int $id { get; }`
    /// When disabled: hook list is always expanded
    ///
    /// Default: true ([PER-CS 4.10](https://www.php-fig.org/per/coding-style/#410-interface-and-abstract-properties) compliant)
    inline_abstract_property_hooks: bool => "default_true",

    /// Whether to add a space before the opening parenthesis in closure use clause.
    ///
    /// When enabled: `function() use ($var)`
    /// When disabled: `function() use($var)`
    ///
    /// Default: true
    space_before_closure_use_clause_parenthesis: bool => "default_true",

    /// Whether to add a space after cast operators (int, float, string, etc.).
    ///
    /// When enabled: `(int) $foo`
    /// When disabled: `(int)$foo`
    ///
    /// Default: true
    space_after_cast_unary_prefix_operators: bool => "default_true",

    /// Whether to add a space after the reference operator (&).
    ///
    /// When enabled: `& $foo`
    /// When disabled: `&$foo`
    ///
    /// Default: false
    space_after_reference_unary_prefix_operator: bool => "default_false",

    /// Whether to add a space after the error control operator (@).
    ///
    /// When enabled: `@ $foo`
    /// When disabled: `@$foo`
    ///
    /// Default: false
    space_after_error_control_unary_prefix_operator: bool => "default_false",

    /// Whether to add a space after the logical not operator (!).
    ///
    /// When enabled: `! $foo`
    /// When disabled: `!$foo`
    ///
    /// Default: false
    space_after_logical_not_unary_prefix_operator: bool => "default_false",

    /// Whether to add a space after the bitwise not operator (~).
    ///
    /// When enabled: `~ $foo`
    /// When disabled: `~$foo`
    ///
    /// Default: false
    space_after_bitwise_not_unary_prefix_operator: bool => "default_false",

    /// Whether to add a space after the increment prefix operator (++).
    ///
    /// When enabled: `++ $i`
    /// When disabled: `++$i`
    ///
    /// Default: false
    space_after_increment_unary_prefix_operator: bool => "default_false",

    /// Whether to add a space after the decrement prefix operator (--).
    ///
    /// When enabled: `-- $i`
    /// When disabled: `--$i`
    ///
    /// Default: false
    space_after_decrement_unary_prefix_operator: bool => "default_false",

    /// Whether to add a space after the additive unary operators (+ and -).
    ///
    /// When enabled: `+ $i`
    /// When disabled: `+$i`
    ///
    /// Default: false
    space_after_additive_unary_prefix_operator: bool => "default_false",

    /// Whether to add spaces around the concatenation operator (.)
    ///
    /// When enabled: `$a . $b`
    /// When disabled: `$a.$b`
    ///
    /// Default: true
    space_around_concatenation_binary_operator: bool => "default_true",

    /// Whether to add spaces around the assignment in declare statements.
    ///
    /// When enabled: `declare(strict_types = 1)`
    /// When disabled: `declare(strict_types=1)`
    ///
    /// Default: false
    space_around_assignment_in_declare: bool => "default_false",

    /// Whether to add spaces within grouping parentheses.
    ///
    /// When enabled: `( $expr ) - $expr`
    /// When disabled: `($expr) - $expr`
    ///
    /// Default: false
    space_within_grouping_parenthesis: bool => "default_false",

    /// Whether to add an empty line after control structures (if, for, foreach, while, do, switch).
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    empty_line_after_control_structure: bool => "default_false",

    /// Whether to add an empty line after opening tag.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    empty_line_after_opening_tag: bool => "default_true",

    /// Whether to add an empty line after declare statement.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    empty_line_after_declare: bool => "default_true",

    /// Whether to add an empty line after namespace.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    empty_line_after_namespace: bool => "default_true",

    /// Whether to add an empty line after use statements.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    empty_line_after_use: bool => "default_true",

    /// Whether to add an empty line after symbols (class, enum, interface, trait, function, const).
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    empty_line_after_symbols: bool => "default_true",

    /// Whether to add an empty line between consecutive symbols of the same type.
    ///
    /// Only applies when `empty_line_after_symbols` is true.
    ///
    /// Default: true
    empty_line_between_same_symbols: bool => "default_true",

    /// Whether to add an empty line after class-like constant.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    empty_line_after_class_like_constant: bool => "default_false",

    /// Whether to add an empty line immediately after a class-like opening brace.
    ///
    /// Default: false
    empty_line_after_class_like_open: bool => "default_false",

    /// Whether to insert an empty line before the closing brace of class-like
    /// structures when the class body is not empty.
    ///
    /// When enabled, a blank line will be inserted immediately before the `}`
    /// that closes a class, trait, interface or enum, but only if the body
    /// contains at least one member.
    ///
    /// Default: false
    empty_line_before_class_like_close: bool => "default_false",

    /// Whether to add an empty line after enum case.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    empty_line_after_enum_case: bool => "default_false",

    /// Whether to add an empty line after trait use.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    empty_line_after_trait_use: bool => "default_false",

    /// Whether to add an empty line after property.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    empty_line_after_property: bool => "default_false",

    /// Whether to add an empty line after method.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    empty_line_after_method: bool => "default_true",

    /// Whether to add an empty line before return statements.
    ///
    /// Default: false
    empty_line_before_return: bool => "default_false",

    /// Whether to add an empty line before dangling comments.
    ///
    /// Default: true
    empty_line_before_dangling_comments: bool => "default_true",

    /// Whether to separate class-like members of different kinds with a blank line.
    ///
    /// Default: true
    separate_class_like_members: bool => "default_true",

    /// Whether to indent heredoc/nowdoc content.
    ///
    /// Default: true
    indent_heredoc: bool => "default_true",

    /// Whether to print boolean and null literals in upper-case (e.g. `TRUE`, `FALSE`, `NULL`).
    /// When enabled these literals are printed in uppercase; when disabled they are printed
    /// in lowercase.
    ///
    /// Default: false
    uppercase_literal_keyword: bool => "default_false",
}

impl Default for FormatSettings {
    /// Sets default values to align with best practices.
    ///
    /// This uses the default preset from the presets module to ensure consistency.
    fn default() -> Self {
        FormatterPreset::Default.settings()
    }
}

/// Specifies the style of line endings.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, JsonSchema)]
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

/// Specifies brace placement style for various constructs.
///
/// - `SameLine`: Opening brace on the same line as the declaration
/// - `NextLine`: Opening brace on the next line for single-line signatures;
///   on the same line when the signature breaks across multiple lines
/// - `AlwaysNextLine`: Opening brace always on the next line, regardless of
///   whether the signature breaks
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, JsonSchema)]
pub enum BraceStyle {
    #[serde(alias = "same_line", alias = "same-line")]
    SameLine,
    #[serde(alias = "next_line", alias = "next-line")]
    NextLine,
    #[serde(alias = "always_next_line", alias = "always-next-line")]
    AlwaysNextLine,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, JsonSchema)]
pub enum MethodChainBreakingStyle {
    #[serde(alias = "same_line", alias = "same-line")]
    SameLine,
    #[default]
    #[serde(alias = "next_line", alias = "next-line")]
    NextLine,
}

impl BraceStyle {
    #[must_use]
    pub fn same_line() -> Self {
        Self::SameLine
    }

    #[must_use]
    pub fn next_line() -> Self {
        Self::NextLine
    }

    #[must_use]
    pub fn always_next_line() -> Self {
        Self::AlwaysNextLine
    }

    #[inline]
    #[must_use]
    pub fn is_next_line(&self) -> bool {
        *self == Self::NextLine
    }

    #[inline]
    #[must_use]
    pub fn is_always_next_line(&self) -> bool {
        *self == Self::AlwaysNextLine
    }
}

impl MethodChainBreakingStyle {
    #[inline]
    #[must_use]
    pub fn is_next_line(&self) -> bool {
        *self == Self::NextLine
    }
}

impl EndOfLine {
    #[inline]
    #[must_use]
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
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, JsonSchema)]
pub enum NullTypeHint {
    #[serde(alias = "null_pipe", alias = "pipe", alias = "long", alias = "|")]
    NullPipe,
    #[default]
    #[serde(alias = "question", alias = "short", alias = "?")]
    Question,
}

impl NullTypeHint {
    #[must_use]
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

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistent_default() {
        let default_settings = FormatSettings::default();
        let default_deserialized: FormatSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(default_settings, default_deserialized);
    }
}
