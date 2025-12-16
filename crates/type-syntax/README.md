# Mago Type Syntax

A fast, memory-efficient Rust crate for parsing PHP docblock type strings (e.g., from `@var`, `@param`, `@return` tags) into a structured Abstract Syntax Tree (AST).

Originally developed as part of the [Mago](https://mago.carthage.software) static analysis toolset, this crate provides the specialized lexer, parser, and AST definitions needed to work with PHP's docblock type syntax, including many Psalm and `PHPStan` extensions.

## Features

- **Dedicated Lexer & Parser:** Includes a performant lexer (`lexer::TypeLexer`) and recursive descent parser (`parser::construct` internally, exposed via `parse_str`) specifically designed for type strings.
- **Structured AST:** Produces a detailed Abstract Syntax Tree (`ast::Type`) representing the type's structure, moving beyond simple string manipulation.
- **Accurate Spans:** Preserves accurate source location (`mago_span::Span`) information for all AST nodes, relative to the original source file (requires providing the correct initial `Span` when parsing).
- **Performance:** Designed with performance and memory efficiency in mind.
- **Error Reporting:** Provides structured error types (`error::ParseError`) with span information on failure.
- **Core Utilities:** Relies on `mago_syntax_core` for shared low-level lexing infrastructure like the `Input` buffer and utility functions/macros.

## Supported Syntax (Examples)

This parser covers a wide range of standard `PHPDoc`, `PHPStan`, and Psalm type syntaxes:

- **Keywords:** `int`, `string`, `bool`, `float`, `mixed`, `null`, `void`, `never`, `object`, `resource`, `true`, `false`, `scalar`, `numeric`, `array-key`, `list`, `non-empty-list`, `non-empty-string`, `class-string`, `iterable`, `callable`, `pure-callable`, `pure-closure`, `stringable-object`, `lowercase-string`, `positive-int`, `negative-int`, `resource`, `closed-resource`, `open-resource`, `numeric-string`, `truthy-string`, etc.
- **Literals:**
  - Strings: `'string-literal'`, `"another one"`
  - Integers: `123`, `-45`, `0x1A`, `0o77`, `0b10`, `123_456`
  - Floats: `1.23`, `-0.5`, `.5`, `1.2e3`, `7E-10`
- **Unspecified Literals:** `literal-int`, `literal-string`, `literal-float`, `non-empty-literal-string`
- **Operators:** `|` (Union), `&` (Intersection), `?` (Nullable)
- **Structure:**
  - Parentheses: `(int|string)`
  - Nullables: `?int`, `?array<string>`
  * Unions: `int|string|null`
  * Intersections: `Countable&Traversable`
  * Member References: `MyClass::CONST`, `MyClass::class`
- **Generics:**
  - `array<KeyType, ValueType>`, `array<ValueType>`
  - `list<ValueType>`, `non-empty-list<ValueType>`
  - `iterable<KeyType, ValueType>`, `iterable<ValueType>`
  - `class-string<ClassName>`, `interface-string<InterfaceName>`, etc.
  - User types: `My\Collection<ItemType>`
  - `self`, `static`, `parent` (Parsed as `Type::Reference` which can have generics)
- **Array Shapes:**
  - `array{key: Type, 'other-key': Type}`
  - `list{Type, Type}`
  - Optional keys: `array{name: string, age?: int}`
  - Unsealed shapes: `array{name: string, ...}`, `list{int, ...<int|string>}`
  - _(Note: Supports any parsed `Type` as a key, per design choice)_
- **Callables:**
  - `callable`, `Closure`, `pure-callable`, `pure-Closure`
  - `callable(ParamType1, ParamType2): ReturnType`
  - `Closure(): void`
  - Optional params: `callable(int=)`
  - Variadic params: `callable(string...)`
- **Variables:** `$var`
- **Conditionals:**
  - `$var is string ? int : bool`
  - `T is not null ? T : mixed`
- **`KeyOf` / `ValueOf`:** `key-of<T>`, `value-of<T>`
- **Indexed Access:** `T[K]`
- **Int Ranges:** `int<0, 100>`, `int<min, 0>`, `int<1, max>`
- **Int Masks:** `int-mask<1, 2, 4>`, `int-mask-of<Permissions::*>`
- **Properties Of:** `properties-of<T>`, `public-properties-of<T>`, `protected-properties-of<T>`, `private-properties-of<T>`
- **Unary `+`/`-` Types:** `+1`, `-2.0` (parsed as `Type::Posited`, `Type::Negated`)

## Usage

1.  **Add Dependencies:**

    Add `mago_type_syntax` to your `Cargo.toml`. You will also likely need `mago_span` and `mago_database` to create the necessary inputs.

    ```toml
    [dependencies]
    mago_type_syntax = "..."
    mago_span = "..."
    mago_database = "..."
    ```

2.  **Parse a Type String:**
    Use the main entry point `mago_type_syntax::parse_str`. You need the type string itself and the `Span` indicating its position within the original source file.

    ```rust,ignore
    use mago_type_syntax::{parse_str, ast::Type};
    use mago_span::{Position, Span};
    use mago_span::HasSpan;
    use mago_database::file::FileId;

    fn main() {
        let type_string = "array<int, string>|null";
        let file_id = FileId::zero(); // Use your actual source identifier

        // Calculate the span of the type string within its original file
        // Example: if it starts at byte 100 and ends at byte 124
        let start_pos = Position::new(file_id, 100);
        let end_pos = Position::new(file_id, 100 + type_string.len());
        let type_span = Span::new(start_pos, end_pos);

        // Parse the string
        match parse_str(type_span, type_string) {
            Ok(parsed_ast) => {
                println!("Successfully parsed AST: {:#?}", parsed_ast);

                // You can now traverse or analyze the parsed_ast (Type enum)
                match parsed_ast {
                    Type::Union(union_type) => {
                        // ... process union ...
                        println!("Parsed a union type!");
                    }
                    Type::Array(array_type) => {
                        // This won't be hit for the example above
                        println!("Parsed an array type!");
                    }
                    // ... handle other Type variants ...
                    _ => { println!("Parsed other type variant"); }
                }
            }
            Err(parse_error) => {
                eprintln!("Failed to parse type string: {:?}", parse_error);
                // Access span via parse_error.span() if needed from HasSpan trait
                eprintln!("Error occurred at span: {:?}", parse_error.span());
            }
        }
    }
    ```
