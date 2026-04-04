# Mago PHPDoc Parser

This crate provides a parser for PHPDoc comments, offering a standardized approach to interpreting PHPDoc blocks.
Given the lack of a strict standard for PHPDoc formatting, we've established our own conventions to ensure consistent parsing and to facilitate tooling such as linters or documentation generators.

## Table of Contents

- [Features](#features)
- [Standardization of PHPDoc Comments](#standardization-of-phpdoc-comments)
  - [Comment Structure](#comment-structure)
    - [Single-Line Comments](#single-line-comments)
    - [Multi-Line Comments](#multi-line-comments)
- [Parsing Elements](#parsing-elements)
  - [Text](#text)
    - [Inline Code](#inline-code)
    - [Inline Tags](#inline-tags)
  - [Code Blocks](#code-blocks)
    - [Fenced Code Blocks](#fenced-code-blocks)
    - [Indented Code Blocks](#indented-code-blocks)
  - [Tags](#tags)
    - [Tag Syntax](#tag-syntax)
    - [Tag Metadata](#tag-metadata)
    - [Tag Parsing](#tag-parsing)
  - [Error Handling](#error-handling)
- [Usage](#usage)
- [Conclusion](#conclusion)

## Features

- Parses PHPDoc comments into a structured AST (Abstract Syntax Tree).
- Supports text, code blocks, tags, inline code, and inline tags.
- Enforces a consistent formatting standard for PHPDoc comments.
- Provides detailed error handling with helpful messages and suggestions.
- Supports tag metadata in parentheses (e.g., `@todo(azjezz) fix this`).

## Standardization of PHPDoc Comments

Due to the absence of an official standard for PHPDoc formatting, we've established conventions to ensure consistent parsing.
This standardization helps in maintaining uniform documentation and facilitates tooling.

### Comment Structure

PHPDoc comments must start with `/**` and end with `*/`. Comments can be single-line or multi-line.

#### Single-line Comments

```php
/** This is a single-line PHPDoc comment */
```

#### Multi-line Comments

```php
/**
 * This is a multi-line PHPDoc comment.
 * Each line starts with an asterisk.
 */
```

## Parsing Elements

The parser recognizes various elements within PHPDoc comments:

### Text

Plain text within the comment is parsed and can include inline code or inline tags.

#### Inline Code

- Inline code is enclosed within backticks `.
- Can be enclosed with one or more backticks (e.g., `` `code` ``, ` ``code`` ` ).
- There must be whitespace before the opening backtick unless it's at the start of a line.

##### Example

```php
/**
 * This is an example of inline code: `echo "Hello, World!";`.
 */
```

#### Inline Tags

- Inline tags are enclosed within `{@` and `}`.
- Used to reference other elements or provide metadata.
- There must be whitespace before the opening `{@` unless it's at the start of a line.

##### Example

```php
/**
 * For more information, see {@see \Some\Class}.
 */
```

### Code Blocks

The parser supports two types of code blocks: fenced code blocks and indented code blocks.

#### Fenced Code Blocks

- Enclosed within triple backticks ` ``` `.
- Optionally, you can specify one or more directives after the opening backticks separated by a comma (e.g., ` ```php,no-run `).
- The code block ends with closing triple backticks.

##### Example

````php
/**
 * This is a fenced code block:
 *
 * ```php
 * echo "Hello, World!";
 * ```
 */
````

#### Indented Code Blocks

- Lines that are indented with spaces or tabs are treated as code blocks.
- The indentation level must be consistent for all lines in the code block.

##### Example

```php
/**
 * This is an indented code block:
 *
 *     echo "Hello, World!";
 */
```

### Tags

Tags provide metadata and additional information about the code. They start with @ and are followed by a tag name.

#### Tag Syntax

- Begins with `@` followed by the tag name.
- The tag name can contain letters, numbers, underscores, hyphens, colons, and backslashes.
- After the tag name, a description can follow, which can span multiple lines until an empty line or another tag/code block is encountered.

##### Example

```php
/**
 * @param string $name The name of the user.
 */
```

#### Tag Parsing

- The parser extracts the tag name.
- Everything following the tag name is considered the description.
- The parser does not interpret the contents of the description; it's treated as plain text.

**Note**: This approach simplifies parsing and delegates the responsibility of interpreting tag descriptions to other tools if necessary.

#### Tag Metadata

Tags can optionally have metadata enclosed in parentheses immediately after the tag name. This supports patterns like:

- `@todo(azjezz) fix this` — metadata is `(azjezz)`
- `@ORM\Entity(repositoryClass="UserRepository")` — metadata is `(repositoryClass="UserRepository")`
- `@Route("/home", name="homepage")` — metadata is `("/home", name="homepage")`

The metadata is captured as a single string (including the parentheses) and is not further parsed. Nested parentheses are handled correctly. If no closing parenthesis is found on the same line, the `(` is treated as part of the description instead.

##### Example

```php
/**
 * @todo(azjezz) Fix the caching logic before release.
 * @ORM\Entity(repositoryClass="App\Repository\UserRepository")
 */
```

### Error Handling

The parser provides detailed error messages and suggestions to help users correct formatting issues. Errors include:

- Invalid Trivia: The comment is not recognized as a PHPDoc block.
- Unclosed Inline Tag: An inline tag is missing a closing `}`.
- Unclosed Inline Code: Inline code is missing a closing backtick `` ` ``.
- Unclosed Code Block: A code block is missing a closing delimiter ` ``` `.
- Invalid Tag Name: The tag name contains invalid characters.

Each error includes:

- A description of the problem.
- A note explaining why it is a problem.
- A help message suggesting how to fix it.

## Usage

To use the parser, include the crate in your project and utilize the public API provided.

```rust,ignore
use mago_database::File::FileId;
use mago_atom::ThreadedInterner;
use mago_span::Span;
use mago_docblock::parse_phpdoc_with_span;

const PHPDOC: &str = r#"/**
 * This is a simple description.
 *
 * @param string $name The name of the user.
 * @return void
 */
"#;

pub fn main() {
    let interner = ThreadedInterner::new();

    let span = Span::new(FileId::zero(), Position::new(0), Position::new(phpdoc.len() as u32));

    let result = parse_phpdoc_with_span(&interner, phpdoc, span);

    match result {
        Ok(document) => {
            println!("Document: {:#?}", document);
        }
        Err(err) => {
            eprintln!("Error: {:#?}", err);
        }
    }
}
```

## Conclusion

This parser aims to provide a consistent and reliable way to parse PHPDoc comments, adhering to a standardized format.
By following the conventions outlined above, you can ensure that your PHPDoc comments are well-structured and easily parsed by tooling,
enhancing code readability and maintainability.

> [!IMPORTANT]
>
> This parser focuses on the structural aspects of PHPDoc comments and does not interpret the semantics of tag descriptions or annotation arguments.
> The descriptions are treated as plain text, allowing other tools to process them as needed.
