---
title: Formatter configuration reference
---

# Configuration reference

While **Mago**'s formatter is opinionated and works great out-of-the-box with its PSR-12 compliant defaults, you can customize its behavior in your `mago.toml` file.

All settings go under the `[formatter]` table.

```toml
[formatter]
print-width = 100
use-tabs = true
```

## Configuration options

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `excludes` | `string[]` | `[]` | A list of paths or glob patterns to exclude from formatting. |
| `print-width` | `integer` | `120` | Maximum line length that the printer will wrap on. |
| `tab-width` | `integer` | `4` | Number of spaces per indentation level. |
| `use-tabs` | `boolean` | `false` | Use tabs instead of spaces for indentation. |
| `end-of-line` | `enum("auto", "lf", "crlf", "cr")` | `"lf"` | The end-of-line character sequence to use. |
| `single-quote` | `boolean` | `true` | Prefer single quotes over double quotes for strings. |
| `trailing-comma` | `boolean` | `true` | Add a trailing comma to multi-line arrays, parameter lists, etc. |
| `remove-trailing-close-tag` | `boolean` | `true` | Remove the trailing PHP close tag (`?>`) from files. |
| `control-brace-style` | `enum("same-line", "next-line")` | `"same-line"` | Brace style for control structures. |
| `closure-brace-style` | `enum("same-line", "next-line")` | `"same-line"` | Brace style for closures. |
| `function-brace-style` | `enum("same-line", "next-line")` | `"next-line"` | Brace style for functions. |
| `method-brace-style` | `enum("same-line", "next-line")` | `"next-line"` | Brace style for methods. |
| `classlike-brace-style` | `enum("same-line", "next-line")` | `"next-line"` | Brace style for classes, traits, etc. |
| `inline-empty-control-braces` | `boolean` | `false` | Place empty control structure bodies on the same line. |
| `inline-empty-closure-braces` | `boolean` | `true` | Place empty closure bodies on the same line. |
| `inline-empty-function-braces` | `boolean` | `false` | Place empty function bodies on the same line. |
| `inline-empty-method-braces` | `boolean` | `false` | Place empty method bodies on the same line. |
| `inline-empty-constructor-braces` | `boolean` | `true` | Place empty constructor bodies on the same line. |
| `inline-empty-classlike-braces` | `boolean` | `false` | Place empty class-like bodies on the same line. |
| `inline-empty-anonymous-class-braces` | `boolean` | `true` | Place empty anonymous class bodies on the same line. |
| `method-chain-breaking-style`| `enum("same-line", "next-line")` | `"next-line"` | How to break method chains. |
| `preserve-breaking-member-access-chain` | `boolean` | `false` | Preserve existing line breaks in method chains. |
| `preserve-breaking-argument-list` | `boolean` | `false` | Preserve existing line breaks in argument lists. |
| `preserve-breaking-array-like` | `boolean` | `true` | Preserve existing line breaks in array-like structures. |
| `preserve-breaking-parameter-list` | `boolean` | `false` | Preserve existing line breaks in parameter lists. |
| `preserve-breaking-attribute-list` | `boolean` | `false` | Preserve existing line breaks in attribute lists. |
| `preserve-breaking-conditional-expression` | `boolean` | `false` | Preserve existing line breaks in ternary expressions. |
| `break-promoted-properties-list` | `boolean` | `true` | Always break parameter lists with promoted properties. |
| `line-before-binary-operator` | `boolean` | `true` | Place the binary operator on the next line when breaking. |
| `always-break-named-arguments-list` | `boolean` | `true` | Always break named argument lists into multiple lines. |
| `always-break-attribute-named-argument-lists` | `boolean` | `false` | Always break named argument lists in attributes. |
| `array-table-style-alignment` | `boolean` | `true` | Use table-style alignment for arrays. |
| `sort-uses` | `boolean` | `true` | Sort `use` statements alphabetically. |
| `separate-use-types` | `boolean` | `true` | Insert a blank line between different types of `use` statements. |
| `expand-use-groups` | `boolean` | `true` | Expand grouped `use` statements into individual statements. |
| `null-type-hint` | `enum("null-pipe", "question")` | `"null-pipe"` | How to format null type hints (`null\|T` vs `?T`). |
| `parentheses-around-new-in-member-access` | `boolean` | `false` | Add parentheses around `new` in member access (`(new Foo)->bar()`). |
| `parentheses-in-new-expression` | `boolean` | `true` | Add parentheses to `new` expressions without arguments (`new Foo()`). |
| `parentheses-in-exit-and-die` | `boolean` | `true` | Add parentheses to `exit` and `die` constructs. |
| `parentheses-in-attribute` | `boolean` | `false` | Add parentheses to attributes without arguments. |
| `space-before-arrow-function-parameter-list-parenthesis` | `boolean` | `false` | Add a space before arrow function parameters. |
| `space-before-closure-parameter-list-parenthesis` | `boolean` | `true` | Add a space before closure parameters. |
| `space-before-hook-parameter-list-parenthesis` | `boolean` | `false` | Add a space before hook parameters. |
| `space-before-closure-use-clause-parenthesis` | `boolean` | `true` | Add a space before closure `use` parentheses. |
| `space-after-colon-in-enum-backing-type` | `boolean` | `true` | Add a space after the colon in enum backing types. |
| `space-after-cast-unary-prefix-operators` | `boolean` | `true` | Add a space after cast operators like `(int)`. |
| `space-after-reference-unary-prefix-operator` | `boolean` | `false` | Add a space after the reference operator (`&`). |
| `space-after-error-control-unary-prefix-operator` | `boolean` | `false` | Add a space after the error control operator (`@`). |
| `space-after-logical-not-unary-prefix-operator` | `boolean` | `false` | Add a space after the logical not operator (`!`). |
| `space-after-bitwise-not-unary-prefix-operator` | `boolean` | `false` | Add a space after the bitwise not operator (`~`). |
| `space-after-increment-unary-prefix-operator` | `boolean` | `false` | Add a space after the prefix increment operator (`++`). |
| `space-after-decrement-unary-prefix-operator` | `boolean` | `false` | Add a space after the prefix decrement operator (`--`). |
| `space-after-additive-unary-prefix-operator` | `boolean` | `false` | Add a space after unary `+` and `-`. |
| `space-around-concatenation-binary-operator` | `boolean` | `true` | Add spaces around the concatenation operator (`.`). |
| `space-around-assignment-in-declare` | `boolean` | `false` | Add spaces around `=` in `declare` statements. |
| `space-within-grouping-parenthesis` | `boolean` | `false` | Add spaces inside grouping parentheses `( 1 + 2 )`. |
| `empty-line-after-control-structure` | `boolean` | `false` | Add an empty line after control structures. |
| `empty-line-after-opening-tag` | `boolean` | `true` | Add an empty line after the opening `<?php` tag. |
| `empty-line-after-declare` | `boolean` | `true` | Add an empty line after a `declare` statement. |
| `empty-line-after-namespace` | `boolean` | `true` | Add an empty line after a `namespace` declaration. |
| `empty-line-after-use` | `boolean` | `true` | Add an empty line after `use` statement blocks. |
| `empty-line-after-symbols` | `boolean` | `true` | Add an empty line after top-level symbols (class, function, etc.). |
| `empty-line-after-class-like-constant` | `boolean` | `false` | Add an empty line after a class constant. |
| `empty-line-after-enum-case` | `boolean` | `false` | Add an empty line after an enum case. |
| `empty-line-after-trait-use` | `boolean` | `false` | Add an empty line after a `use` statement inside a trait. |
| `empty-line-after-property` | `boolean` | `false` | Add an empty line after a property. |
| `empty-line-after-method` | `boolean` | `true` | Add an empty line after a method. |
| `empty-line-before-return` | `boolean` | `false` | Add an empty line before a `return` statement. |
| `empty-line-before-dangling-comments` | `boolean` | `true` | Add an empty line before dangling comments. |
| `separate-class-like-members` | `boolean` | `true` | Separate different kinds of class members with a blank line. |
