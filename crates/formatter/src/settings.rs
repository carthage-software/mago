use std::str::FromStr;

use crate::presets;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

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
    /// Maximum line length that the printer will wrap on.
    ///
    /// Default: 120
    #[serde(default = "default_print_width")]
    pub print_width: usize,

    /// Number of spaces per indentation level.
    ///
    /// Default: 4
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,

    /// Whether to use tabs instead of spaces for indentation.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub use_tabs: bool,

    /// End-of-line characters to use.
    ///
    /// Default: "lf"
    #[serde(default)]
    pub end_of_line: EndOfLine,

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
    #[serde(default = "default_true")]
    pub single_quote: bool,

    /// Whether to add a trailing comma to the last element in multi-line syntactic structures.
    ///
    /// When enabled, trailing commas are added to lists, arrays, parameter lists,
    /// argument lists, and other similar structures when they span multiple lines.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub trailing_comma: bool,

    /// Whether to remove the trailing PHP close tag (`?>`) from files.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub remove_trailing_close_tag: bool,

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
    #[serde(default = "BraceStyle::same_line")]
    pub control_brace_style: BraceStyle,

    /// Whether to place `else`, `elseif`, `catch` and `finally` on a new line.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub following_clause_on_newline: bool,

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
    #[serde(default = "BraceStyle::same_line")]
    pub closure_brace_style: BraceStyle,

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
    #[serde(default = "BraceStyle::next_line")]
    pub function_brace_style: BraceStyle,

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
    #[serde(default = "BraceStyle::next_line")]
    pub method_brace_style: BraceStyle,

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
    #[serde(default = "BraceStyle::always_next_line")]
    pub classlike_brace_style: BraceStyle,

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
    #[serde(default = "default_false")]
    pub inline_empty_control_braces: bool,

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
    #[serde(default = "default_true")]
    pub inline_empty_closure_braces: bool,

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
    /// Default: false
    #[serde(default = "default_false")]
    pub inline_empty_function_braces: bool,

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
    /// Default: false
    #[serde(default = "default_false")]
    pub inline_empty_method_braces: bool,

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
    #[serde(default = "default_true")]
    pub inline_empty_constructor_braces: bool,

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
    #[serde(default = "default_true")]
    pub inline_empty_classlike_braces: bool,

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
    #[serde(default = "default_true")]
    pub inline_empty_anonymous_class_braces: bool,

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
    #[serde(default)]
    pub method_chain_breaking_style: MethodChainBreakingStyle,

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
    #[serde(default = "default_true")]
    pub first_method_chain_on_new_line: bool,

    /// Whether to preserve line breaks in method chains, even if they could fit on a single line.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub preserve_breaking_member_access_chain: bool,

    /// Whether to preserve line breaks in argument lists, even if they could fit on a single line.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub preserve_breaking_argument_list: bool,

    /// Whether to preserve line breaks in array-like structures, even if they could fit on a single line.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub preserve_breaking_array_like: bool,

    /// Whether to preserve line breaks in parameter lists, even if they could fit on a single line.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub preserve_breaking_parameter_list: bool,

    /// Whether to preserve line breaks in attribute lists, even if they could fit on a single line.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub preserve_breaking_attribute_list: bool,

    /// Whether to preserve line breaks in conditional (ternary) expressions.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub preserve_breaking_conditional_expression: bool,

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
    #[serde(default = "default_true")]
    pub break_promoted_properties_list: bool,

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
    #[serde(default = "default_true")]
    pub line_before_binary_operator: bool,

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
    #[serde(default = "default_false")]
    pub always_break_named_arguments_list: bool,

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
    #[serde(default = "default_false")]
    pub always_break_attribute_named_argument_lists: bool,

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
    #[serde(default = "default_true")]
    pub array_table_style_alignment: bool,

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
    #[serde(default = "default_false")]
    pub align_assignment_like: bool,

    /// Whether to sort use statements alphabetically.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub sort_uses: bool,

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
    #[serde(default = "default_false")]
    pub sort_class_methods: bool,

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
    #[serde(default = "default_true")]
    pub separate_use_types: bool,

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
    #[serde(default = "default_true")]
    pub expand_use_groups: bool,

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
    #[serde(default)]
    pub null_type_hint: NullTypeHint,

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
    #[serde(default = "default_false")]
    pub parentheses_around_new_in_member_access: bool,

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
    #[serde(default = "default_true")]
    pub parentheses_in_new_expression: bool,

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
    #[serde(default = "default_true")]
    pub parentheses_in_exit_and_die: bool,

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
    #[serde(default = "default_false")]
    pub parentheses_in_attribute: bool,

    /// Whether to add a space before the opening parameters in arrow functions.
    ///
    /// When enabled: `fn ($x) => $x * 2`
    /// When disabled: `fn($x) => $x * 2`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_arrow_function_parameter_list_parenthesis: bool,

    /// Whether to add a space before the opening parameters in closures.
    ///
    /// When enabled: `function ($x) use ($y)`
    /// When disabled: `function($x) use ($y)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_closure_parameter_list_parenthesis: bool,

    /// Whether to add a space before the opening parameters in hooks.
    ///
    /// When enabled: `$hook ($param)`
    /// When disabled: `$hook($param)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_hook_parameter_list_parenthesis: bool,

    /// Whether to keep abstract property hooks inline.
    ///
    /// When enabled: `public int $id { get; }`
    /// When disabled: hook list is always expanded
    ///
    /// Default: true ([PER-CS 4.10](https://www.php-fig.org/per/coding-style/#410-interface-and-abstract-properties) compliant)
    #[serde(default = "default_true")]
    pub inline_abstract_property_hooks: bool,

    /// Whether to add a space before the opening parenthesis in closure use clause.
    ///
    /// When enabled: `function() use ($var)`
    /// When disabled: `function() use($var)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_closure_use_clause_parenthesis: bool,

    /// Whether to add a space after cast operators (int, float, string, etc.).
    ///
    /// When enabled: `(int) $foo`
    /// When disabled: `(int)$foo`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_after_cast_unary_prefix_operators: bool,

    /// Whether to add a space after the reference operator (&).
    ///
    /// When enabled: `& $foo`
    /// When disabled: `&$foo`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_after_reference_unary_prefix_operator: bool,

    /// Whether to add a space after the error control operator (@).
    ///
    /// When enabled: `@ $foo`
    /// When disabled: `@$foo`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_after_error_control_unary_prefix_operator: bool,

    /// Whether to add a space after the logical not operator (!).
    ///
    /// When enabled: `! $foo`
    /// When disabled: `!$foo`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_after_logical_not_unary_prefix_operator: bool,

    /// Whether to add a space after the bitwise not operator (~).
    ///
    /// When enabled: `~ $foo`
    /// When disabled: `~$foo`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_after_bitwise_not_unary_prefix_operator: bool,

    /// Whether to add a space after the increment prefix operator (++).
    ///
    /// When enabled: `++ $i`
    /// When disabled: `++$i`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_after_increment_unary_prefix_operator: bool,

    /// Whether to add a space after the decrement prefix operator (--).
    ///
    /// When enabled: `-- $i`
    /// When disabled: `--$i`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_after_decrement_unary_prefix_operator: bool,

    /// Whether to add a space after the additive unary operators (+ and -).
    ///
    /// When enabled: `+ $i`
    /// When disabled: `+$i`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_after_additive_unary_prefix_operator: bool,

    /// Whether to add spaces around the concatenation operator (.)
    ///
    /// When enabled: `$a . $b`
    /// When disabled: `$a.$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_concatenation_binary_operator: bool,

    /// Whether to add spaces around the assignment in declare statements.
    ///
    /// When enabled: `declare(strict_types = 1)`
    /// When disabled: `declare(strict_types=1)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_around_assignment_in_declare: bool,

    /// Whether to add spaces within grouping parentheses.
    ///
    /// When enabled: `( $expr ) - $expr`
    /// When disabled: `($expr) - $expr`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_grouping_parenthesis: bool,

    /// Whether to add an empty line after control structures (if, for, foreach, while, do, switch).
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_after_control_structure: bool,

    /// Whether to add an empty line after opening tag.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_after_opening_tag: bool,

    /// Whether to add an empty line after declare statement.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_after_declare: bool,

    /// Whether to add an empty line after namespace.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_after_namespace: bool,

    /// Whether to add an empty line after use statements.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_after_use: bool,

    /// Whether to add an empty line after symbols (class, enum, interface, trait, function, const).
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_after_symbols: bool,

    /// Whether to add an empty line between consecutive symbols of the same type.
    ///
    /// Only applies when `empty_line_after_symbols` is true.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_between_same_symbols: bool,

    /// Whether to add an empty line after class-like constant.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_after_class_like_constant: bool,

    /// Whether to add an empty line immediately after a class-like opening brace.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_after_class_like_open: bool,

    /// Whether to insert an empty line before the closing brace of class-like
    /// structures when the class body is not empty.
    ///
    /// When enabled, a blank line will be inserted immediately before the `}`
    /// that closes a class, trait, interface or enum, but only if the body
    /// contains at least one member.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_before_class_like_close: bool,

    /// Whether to add an empty line after enum case.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_after_enum_case: bool,

    /// Whether to add an empty line after trait use.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_after_trait_use: bool,

    /// Whether to add an empty line after property.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_after_property: bool,

    /// Whether to add an empty line after method.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_after_method: bool,

    /// Whether to add an empty line before return statements.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_before_return: bool,

    /// Whether to add an empty line before dangling comments.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_before_dangling_comments: bool,

    /// Whether to separate class-like members of different kinds with a blank line.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub separate_class_like_members: bool,

    /// Whether to indent heredoc/nowdoc content.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub indent_heredoc: bool,

    /// Whether to print boolean and null literals in upper-case (e.g. `TRUE`, `FALSE`, `NULL`).
    /// When enabled these literals are printed in uppercase; when disabled they are printed
    /// in lowercase.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub uppercase_literal_keyword: bool,
}

impl Default for FormatSettings {
    /// Sets default values to align with best practices.
    ///
    /// This uses the default preset from the presets module to ensure consistency.
    fn default() -> Self {
        presets::FormatterPreset::Default.settings()
    }
}

/// Merges individual settings over a preset, applying overrides where individual
/// settings differ from the default values (indicating they were explicitly set).
pub fn merge_format_settings(preset: FormatSettings, individual: FormatSettings) -> FormatSettings {
    let defaults = FormatSettings::default();

    macro_rules! merge_fields {
        ($($field:ident),* $(,)?) => {
            FormatSettings {
                $(
                    $field: if individual.$field != defaults.$field {
                        // User explicitly set this value (differs from default)
                        individual.$field
                    } else {
                        // Use preset value
                        preset.$field
                    },
                )*
            }
        };
    }

    merge_fields! {
        print_width,
        tab_width,
        use_tabs,
        end_of_line,
        single_quote,
        trailing_comma,
        remove_trailing_close_tag,
        control_brace_style,
        following_clause_on_newline,
        closure_brace_style,
        function_brace_style,
        method_brace_style,
        classlike_brace_style,
        inline_empty_control_braces,
        inline_empty_closure_braces,
        inline_empty_function_braces,
        inline_empty_method_braces,
        inline_empty_constructor_braces,
        inline_empty_classlike_braces,
        inline_empty_anonymous_class_braces,
        method_chain_breaking_style,
        first_method_chain_on_new_line,
        preserve_breaking_member_access_chain,
        preserve_breaking_argument_list,
        preserve_breaking_array_like,
        preserve_breaking_parameter_list,
        preserve_breaking_attribute_list,
        preserve_breaking_conditional_expression,
        break_promoted_properties_list,
        line_before_binary_operator,
        always_break_named_arguments_list,
        always_break_attribute_named_argument_lists,
        array_table_style_alignment,
        align_assignment_like,
        sort_uses,
        sort_class_methods,
        separate_use_types,
        expand_use_groups,
        null_type_hint,
        parentheses_around_new_in_member_access,
        parentheses_in_new_expression,
        parentheses_in_exit_and_die,
        parentheses_in_attribute,
        space_before_arrow_function_parameter_list_parenthesis,
        space_before_closure_parameter_list_parenthesis,
        space_before_hook_parameter_list_parenthesis,
        inline_abstract_property_hooks,
        space_before_closure_use_clause_parenthesis,
        space_after_cast_unary_prefix_operators,
        space_after_reference_unary_prefix_operator,
        space_after_error_control_unary_prefix_operator,
        space_after_logical_not_unary_prefix_operator,
        space_after_bitwise_not_unary_prefix_operator,
        space_after_increment_unary_prefix_operator,
        space_after_decrement_unary_prefix_operator,
        space_after_additive_unary_prefix_operator,
        space_around_concatenation_binary_operator,
        space_around_assignment_in_declare,
        space_within_grouping_parenthesis,
        empty_line_after_control_structure,
        empty_line_after_opening_tag,
        empty_line_after_declare,
        empty_line_after_namespace,
        empty_line_after_use,
        empty_line_after_symbols,
        empty_line_between_same_symbols,
        empty_line_after_class_like_constant,
        empty_line_after_class_like_open,
        empty_line_before_class_like_close,
        empty_line_after_enum_case,
        empty_line_after_trait_use,
        empty_line_after_property,
        empty_line_after_method,
        empty_line_before_return,
        empty_line_before_dangling_comments,
        separate_class_like_members,
        indent_heredoc,
        uppercase_literal_keyword,
    }
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
