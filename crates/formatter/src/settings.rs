use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

/// Format settings for the PHP printer.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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
    /// Default: same_line
    #[serde(default = "BraceStyle::same_line")]
    pub control_brace_style: BraceStyle,

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
    /// Default: same_line
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
    /// Default: next_line
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
    /// Default: next_line
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
    /// Example with `next_line`:
    /// ```php
    /// class Foo
    /// {
    /// }
    /// ```
    ///
    /// Default: next_line
    #[serde(default = "BraceStyle::next_line")]
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
    /// Default: false
    #[serde(default = "default_false")]
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
    /// Default: next_line
    #[serde(default)]
    pub method_chain_breaking_style: MethodChainBreakingStyle,

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
    /// Default: false
    #[serde(default = "default_false")]
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
    /// Default: true
    #[serde(default = "default_true")]
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

    /// Whether to sort use statements alphabetically.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub sort_uses: bool,

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
    /// With `NullPipe`:
    /// ```php
    /// function foo(null|string $bar) {
    ///     return $bar;
    /// }
    /// ```
    ///
    /// With `Question`:
    /// ```php
    /// function foo(?string $bar) {
    ///     return $bar;
    /// }
    /// ```
    ///
    /// Default: NullPipe
    #[serde(default)]
    pub null_type_hint: NullTypeHint,

    /// Whether to put `static` before visibility keywords.
    ///
    /// When enabled:
    /// ```php
    /// class Foo {
    ///     static public $bar;
    /// }
    /// ```
    ///
    /// When disabled:
    /// ```php
    /// class Foo {
    ///     public static $bar;
    /// }
    /// ```
    ///
    /// This also affects placement of the `readonly` keyword.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub static_before_visibility: bool,

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

    /// Whether to add a space before the opening parenthesis in `if` statements.
    ///
    /// When enabled: `if ($condition)`
    /// When disabled: `if($condition)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_if_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in `for` loops.
    ///
    /// When enabled: `for ($i = 0; $i < 10; $i++)`
    /// When disabled: `for($i = 0; $i < 10; $i++)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_for_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in `foreach` loops.
    ///
    /// When enabled: `foreach ($items as $item)`
    /// When disabled: `foreach($items as $item)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_foreach_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in `while` loops.
    ///
    /// When enabled: `while ($condition)`
    /// When disabled: `while($condition)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_while_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in `catch` blocks.
    ///
    /// When enabled: `catch (Exception $e)`
    /// When disabled: `catch(Exception $e)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_catch_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in `switch` statements.
    ///
    /// When enabled: `switch ($value)`
    /// When disabled: `switch($value)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_switch_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in `match` expressions.
    ///
    /// When enabled: `match ($value)`
    /// When disabled: `match($value)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_match_parenthesis: bool,

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

    /// Whether to add a space before the opening parameters in function declarations.
    ///
    /// When enabled: `function foo ($param)`
    /// When disabled: `function foo($param)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_function_parameter_list_parenthesis: bool,

    /// Whether to add a space before the opening parameters in method declarations.
    ///
    /// When enabled: `public function foo ($param)`
    /// When disabled: `public function foo($param)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_method_parameter_list_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in closure use clause.
    ///
    /// When enabled: `function() use ($var)`
    /// When disabled: `function() use($var)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_before_closure_use_clause_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in function-like calls.
    ///
    /// When enabled: `foo ($arg)`
    /// When disabled: `foo($arg)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_argument_list_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in list destructuring.
    ///
    /// When enabled: `list ($a, $b) = $array`
    /// When disabled: `list($a, $b) = $array`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_list_parenthesis: bool,

    /// Whether to add a space before the opening parenthesis in legacy array syntax.
    ///
    /// When enabled: `array ($item)`
    /// When disabled: `array($item)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_legacy_array_parenthesis: bool,

    /// Whether to add a space before increment postfix operator (++).
    ///
    /// When enabled: `$i ++`
    /// When disabled: `$i++`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_increment_unary_postfix_operator: bool,

    /// Whether to add a space before decrement postfix operator (--).
    ///
    /// When enabled: `$i --`
    /// When disabled: `$i--`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_decrement_unary_postfix_operator: bool,

    /// Whether to add a space before semicolons in for loops.
    ///
    /// When enabled: `for ($i = 0 ; $i < 10 ; $i++)`
    /// When disabled: `for ($i = 0; $i < 10; $i++)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_for_semicolon: bool,

    /// Whether to add a space before the colon in return type hints.
    ///
    /// When enabled: `function foo() : string`
    /// When disabled: `function foo(): string`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_colon_in_return_type: bool,

    /// Whether to add a space before the colon in named arguments.
    ///
    /// When enabled: `foo(name : 'value')`
    /// When disabled: `foo(name: 'value')`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_colon_in_named_argument: bool,

    /// Whether to add a space before the colon in enum backing types.
    ///
    /// When enabled: `enum Foo : string`
    /// When disabled: `enum Foo: string`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_before_colon_in_enum_backing_type: bool,

    /// Whether to add a space after semicolons in for loops.
    ///
    /// When enabled: `for ($i = 0; $i < 10; $i++)`
    /// When disabled: `for ($i = 0;$i < 10;$i++)`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_after_for_semicolon: bool,

    /// Whether to add a space after the colon in return type hints.
    ///
    /// When enabled: `function foo(): string`
    /// When disabled: `function foo():string`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_after_colon_in_return_type: bool,

    /// Whether to add a space after the colon in named arguments.
    ///
    /// When enabled: `foo(name: 'value')`
    /// When disabled: `foo(name:'value')`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_after_colon_in_named_argument: bool,

    /// Whether to add a space after the colon in enum backing types.
    ///
    /// When enabled: `enum Foo: string`
    /// When disabled: `enum Foo:string`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_after_colon_in_enum_backing_type: bool,

    /// Whether to add a space after the question mark in nullable type hints.
    ///
    /// When enabled: `function foo(? string $bar)`
    /// When disabled: `function foo(?string $bar)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_after_nullable_type_question_mark: bool,

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

    /// Whether to add spaces around assignment operators.
    ///
    /// When enabled: `$a = $b`
    /// When disabled: `$a=$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_assignment_operators: bool,

    /// Whether to add spaces around logical operators (&&, ||, and, or, xor).
    ///
    /// When enabled: `$a && $b`
    /// When disabled: `$a&&$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_logical_binary_operators: bool,

    /// Whether to add spaces around equality operators (==, ===, !=, !==).
    ///
    /// When enabled: `$a === $b`
    /// When disabled: `$a===$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_equality_binary_operators: bool,

    /// Whether to add spaces around comparison operators (<, >, <=, >=, <=>).
    ///
    /// When enabled: `$a < $b`
    /// When disabled: `$a<$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_comparison_binary_operators: bool,

    /// Whether to add spaces around bitwise operators (&, |, ^).
    ///
    /// When enabled: `$a | $b`
    /// When disabled: `$a|$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_bitwise_binary_operators: bool,

    /// Whether to add spaces around additive operators (+ and -).
    ///
    /// When enabled: `$a + $b`
    /// When disabled: `$a+$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_additive_binary_operators: bool,

    /// Whether to add spaces around multiplicative operators (*, /, %).
    ///
    /// When enabled: `$a * $b`
    /// When disabled: `$a*$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_multiplicative_binary_operators: bool,

    /// Whether to add spaces around exponentiation operator (**).
    ///
    /// When enabled: `$a ** $b`
    /// When disabled: `$a**$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_exponentiation_binary_operators: bool,

    /// Whether to add spaces around shift operators (<<, >>).
    ///
    /// When enabled: `$a << $b`
    /// When disabled: `$a<<$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_shift_binary_operators: bool,

    /// Whether to add spaces around the concatenation operator (.)
    ///
    /// When enabled: `$a . $b`
    /// When disabled: `$a.$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_concatenation_binary_operator: bool,

    /// Whether to add spaces around the null coalescing operator (??).
    ///
    /// When enabled: `$a ?? $b`
    /// When disabled: `$a??$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_null_coalescing_binary_operator: bool,

    /// Whether to add spaces around the elvis operator (?:).
    ///
    /// When enabled: `$a ?: $b`
    /// When disabled: `$a?:$b`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub space_around_elvis_binary_operator: bool,

    /// Whether to add spaces around the assignment in declare statements.
    ///
    /// When enabled: `declare(strict_types = 1)`
    /// When disabled: `declare(strict_types=1)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_around_assignment_in_declare: bool,

    /// Whether to add spaces around the pipe in union type hints.
    ///
    /// When enabled: `function foo(int | string $bar)`
    /// When disabled: `function foo(int|string $bar)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_around_pipe_in_union_type: bool,

    /// Whether to add spaces around the pipe in union type hints.
    ///
    /// When enabled: `function foo(Foo & Bar $bar)`
    /// When disabled: `function foo(Foo&Bar $bar)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_around_ampersand_in_intersection_type: bool,

    /// Whether to add spaces within array access brackets.
    ///
    /// When enabled: `$array[ $key ]`
    /// When disabled: `$array[$key]`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_array_access_brackets: bool,

    /// Whether to add spaces within type parentheses.
    ///
    /// When enabled: `function foo(string|( Foo&Stringable ) $bar)`
    /// When disabled: `function foo(string|(Foo&Stringable) $bar)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_type_parenthesis: bool,

    /// Whether to add spaces within array brackets.
    ///
    /// When enabled: `[ $item1, $item2 ]`
    /// When disabled: `[$item1, $item2]`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_array_brackets: bool,

    /// Whether to add spaces within grouping parentheses.
    ///
    /// When enabled: `( $expr ) - $expr`
    /// When disabled: `($expr) - $expr`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_grouping_parenthesis: bool,

    /// Whether to add spaces within legacy array parentheses.
    ///
    /// When enabled: `array( $item1, $item2 )`
    /// When disabled: `array($item1, $item2)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_legacy_array_parenthesis: bool,

    /// Whether to add spaces within list parentheses.
    ///
    /// When enabled: `list( $item1, $item2 )`
    /// When disabled: `list($item1, $item2)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_list_parenthesis: bool,

    /// Whether to add spaces within argument list parentheses.
    ///
    /// When enabled: `some_function( $arg1, $arg2 )`
    /// When disabled: `some_function($arg1, $arg2)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_argument_list_parenthesis: bool,

    /// Whether to add spaces within parameter list parentheses.
    ///
    /// When enabled: `function foo( $param1, $param2 )`
    /// When disabled: `function foo($param1, $param2)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_parameter_list_parenthesis: bool,

    /// Whether to add spaces within if statement parentheses.
    ///
    /// When enabled: `if ( $condition )`
    /// When disabled: `if ($condition)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_if_parenthesis: bool,

    /// Whether to add spaces within for loop parentheses.
    ///
    /// When enabled: `for ( $i = 0; $i < 10; $i++ )`
    /// When disabled: `for ($i = 0; $i < 10; $i++)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_for_parenthesis: bool,

    /// Whether to add spaces within foreach loop parentheses.
    ///
    /// When enabled: `foreach ( $items as $item )`
    /// When disabled: `foreach ($items as $item)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_foreach_parenthesis: bool,

    /// Whether to add spaces within while loop parentheses.
    ///
    /// When enabled: `while ( $condition )`
    /// When disabled: `while ($condition)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_while_parenthesis: bool,

    /// Whether to add spaces within catch block parentheses.
    ///
    /// When enabled: `catch ( Throwable $exception )`
    /// When disabled: `catch (Throwable $exception)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_catch_parenthesis: bool,

    /// Whether to add spaces within switch statement parentheses.
    ///
    /// When enabled: `switch ( $value )`
    /// When disabled: `switch ($value)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_switch_parenthesis: bool,

    /// Whether to add spaces within match statement parentheses.
    ///
    /// When enabled: `match ( $value )`
    /// When disabled: `match ($value)`
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub space_within_match_parenthesis: bool,

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

    /// Whether to add an empty line after class-like constant.
    ///
    /// Note: if an empty line already exists, it will be preserved regardless of this
    /// settings value.
    ///
    /// Default: false
    #[serde(default = "default_false")]
    pub empty_line_after_class_like_constant: bool,

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
    /// Default: true
    #[serde(default = "default_false")]
    pub empty_line_before_return: bool,

    /// Whether to add an empty line before return statements.
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub empty_line_before_dangling_comments: bool,

    /// Whether to use double slash comments instead of hash comments.
    ///
    /// When enabled: `// This is a comment`
    /// When disabled: `# This is a comment`
    ///
    /// Default: true
    #[serde(default = "default_true")]
    pub double_slash_comments: bool,
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
            closure_brace_style: BraceStyle::SameLine,
            function_brace_style: BraceStyle::NextLine,
            method_brace_style: BraceStyle::NextLine,
            classlike_brace_style: BraceStyle::NextLine,
            control_brace_style: BraceStyle::SameLine,
            inline_empty_control_braces: false,
            inline_empty_closure_braces: true,
            inline_empty_function_braces: false,
            inline_empty_method_braces: false,
            inline_empty_constructor_braces: true,
            inline_empty_classlike_braces: false,
            inline_empty_anonymous_class_braces: true,
            static_before_visibility: false,
            null_type_hint: NullTypeHint::default(),
            break_promoted_properties_list: true,
            method_chain_breaking_style: MethodChainBreakingStyle::NextLine,
            line_before_binary_operator: false,
            sort_uses: true,
            separate_use_types: true,
            expand_use_groups: true,
            remove_trailing_close_tag: true,
            parentheses_around_new_in_member_access: false,
            parentheses_in_new_expression: true,
            parentheses_in_exit_and_die: true,
            parentheses_in_attribute: false,
            array_table_style_alignment: true,
            always_break_named_arguments_list: true,
            always_break_attribute_named_argument_lists: false,
            preserve_breaking_member_access_chain: false,
            preserve_breaking_argument_list: false,
            preserve_breaking_array_like: true,
            preserve_breaking_parameter_list: false,
            preserve_breaking_attribute_list: false,
            preserve_breaking_conditional_expression: false,
            space_before_if_parenthesis: true,
            space_before_for_parenthesis: true,
            space_before_foreach_parenthesis: true,
            space_before_while_parenthesis: true,
            space_before_catch_parenthesis: true,
            space_before_switch_parenthesis: true,
            space_before_match_parenthesis: true,
            space_before_arrow_function_parameter_list_parenthesis: false,
            space_before_closure_parameter_list_parenthesis: true,
            space_before_closure_use_clause_parenthesis: true,
            space_before_argument_list_parenthesis: false,
            space_before_function_parameter_list_parenthesis: false,
            space_before_list_parenthesis: false,
            space_before_legacy_array_parenthesis: false,
            space_before_for_semicolon: false,
            space_before_colon_in_return_type: false,
            space_before_colon_in_named_argument: false,
            space_before_colon_in_enum_backing_type: false,
            space_around_assignment_in_declare: false,
            space_around_pipe_in_union_type: false,
            space_within_array_access_brackets: false,
            space_within_array_brackets: false,
            space_within_grouping_parenthesis: false,
            space_within_list_parenthesis: false,
            space_within_argument_list_parenthesis: false,
            space_within_parameter_list_parenthesis: false,
            space_within_if_parenthesis: false,
            space_within_for_parenthesis: false,
            space_within_foreach_parenthesis: false,
            space_within_while_parenthesis: false,
            space_within_catch_parenthesis: false,
            space_within_switch_parenthesis: false,
            space_within_match_parenthesis: false,
            space_after_for_semicolon: true,
            space_after_colon_in_return_type: true,
            space_after_colon_in_named_argument: true,
            space_after_colon_in_enum_backing_type: true,
            space_before_hook_parameter_list_parenthesis: false,
            space_before_method_parameter_list_parenthesis: false,
            space_before_increment_unary_postfix_operator: false,
            space_before_decrement_unary_postfix_operator: false,
            space_around_assignment_operators: true,
            space_around_logical_binary_operators: true,
            space_around_equality_binary_operators: true,
            space_around_comparison_binary_operators: true,
            space_around_bitwise_binary_operators: true,
            space_around_additive_binary_operators: true,
            space_around_multiplicative_binary_operators: true,
            space_around_exponentiation_binary_operators: true,
            space_around_shift_binary_operators: true,
            space_around_concatenation_binary_operator: true,
            space_around_null_coalescing_binary_operator: true,
            space_around_elvis_binary_operator: true,
            space_around_ampersand_in_intersection_type: false,
            space_within_type_parenthesis: false,
            space_within_legacy_array_parenthesis: false,
            space_after_cast_unary_prefix_operators: true,
            space_after_reference_unary_prefix_operator: false,
            space_after_error_control_unary_prefix_operator: false,
            space_after_logical_not_unary_prefix_operator: false,
            space_after_bitwise_not_unary_prefix_operator: false,
            space_after_increment_unary_prefix_operator: false,
            space_after_decrement_unary_prefix_operator: false,
            space_after_additive_unary_prefix_operator: false,
            space_after_nullable_type_question_mark: false,
            empty_line_after_control_structure: false,
            empty_line_after_opening_tag: false,
            empty_line_after_declare: true,
            empty_line_after_namespace: true,
            empty_line_after_use: true,
            empty_line_after_symbols: true,
            empty_line_after_class_like_constant: false,
            empty_line_after_enum_case: false,
            empty_line_after_trait_use: false,
            empty_line_after_property: false,
            empty_line_after_method: true,
            empty_line_before_return: false,
            empty_line_before_dangling_comments: true,
            double_slash_comments: true,
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

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}
