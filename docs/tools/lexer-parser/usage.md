---
title: Lexer and parser usage
---

# Using Mago's lexer and parser

The primary way to interact with Mago's parser is through the `mago ast` command. It takes a single PHP file and can output its structure in several different formats.

Let's consider a simple file, `example.php`:

```php
<?php

echo 'Hello, World!';
```

### Default tree view

By default, `mago ast` prints a human-readable tree that visualizes the Abstract Syntax Tree (AST).

```sh
mago ast example.php
```

This will produce an output like this, showing the nested structure of the code:

```sh
Program
├── Statement
│   └── OpeningTag
│       └── FullOpeningTag
└── Statement
    └── Echo
        ├── Keyword
        ├── Expression
        │   └── Literal
        │       └── LiteralString "Hello, World!"
        └── Terminator ;
```

### Token view

To see the raw token stream from the lexer, use the `--tokens` flag. This is useful for debugging low-level syntax issues.

```sh
mago ast example.php --tokens
```

This will output a table of all tokens found in the file:

```sh
  Kind                      Value                                              Span
  ───────────────────────── ────────────────────────────────────────────────── ────────────────────
  OpenTag                   "<?php"                                            [0..5]
  Whitespace                "\n\n"                                             [7..7]
  Echo                      "echo"                                             [7..11]
  Whitespace                " "                                                [12..12]
  LiteralString             "'Hello, World!'"                                  [12..27]
  Semicolon                 ";"                                                [27..28]
  Whitespace                "\n"                                               [29..29]
```

### JSON output

For machine-readable output, you can combine the `--json` flag with either the default AST view or the `--tokens` view. This is perfect for scripting or for other tools to consume Mago's output.

```sh
mago ast example.php --json
```

This will produce a detailed JSON object representing the full AST.

```json
{
  "error": null,
  "program": {
    "file_id": 9370985751100973094,
    "source_text": "<?php\n\necho 'Hello, World!';\n",
    "statements": {
      "nodes": [
        // ...
      ]
    },
    "trivia": {
      "nodes": [
        // ...
      ]
    }
  }
}
```

For more details on the available command-line options, see the [Command Reference](/tools/lexer-parser/command-reference.md).
