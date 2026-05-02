+++
title = "Inspecting the AST"
description = "The mago ast command for poking at the parser output. A debugging tool, not part of the regular workflow."
nav_order = 75
nav_section = "Guide"
+++
# Inspecting the AST

`mago ast` prints either the abstract syntax tree or the token stream for a single PHP file. Useful for debugging a tricky parse, understanding how Mago sees a piece of syntax, or feeding the output to another tool that wants Mago's parser. It is not part of the regular formatter / linter / analyzer / guard workflow; treat it as a debug command, alongside `list-files` and friends.

## Tree view

Given `example.php`:

```php
<?php

echo 'Hello, World!';
```

```sh
mago ast example.php
```

```
Program
├── Statement
│ └── OpeningTag
│ └── FullOpeningTag
└── Statement
 └── Echo
 ├── Keyword
 ├── Expression
 │ └── Literal
 │ └── LiteralString "Hello, World!"
 └── Terminator ;
```

## Token view

`--tokens` prints the lexer's token stream instead. Useful for debugging low-level syntax issues.

```sh
mago ast example.php --tokens
```

```
 Kind                      Value                                              Span
 ─────────────────────────────────────────────────────────────────────────────────────────────
 OpenTag                   "<?php"                                            [0..5]
 Whitespace                "\n\n"                                             [7..7]
 Echo                      "echo"                                             [7..11]
 Whitespace                " "                                                [12..12]
 LiteralString             "'Hello, World!'"                                  [12..27]
 Semicolon                 ";"                                                [27..28]
 Whitespace                "\n"                                               [29..29]
```

## JSON output

`--json` switches either view to pretty-printed JSON. Combine with `--tokens` for a token-stream JSON, or alone for the full AST.

```sh
mago ast example.php --json
```

```json
{
    "error": null,
    "program": {
        "file_id": 9370985751100973094,
        "source_text": "<?php\n\necho 'Hello, World!';\n",
        "statements": { "nodes": [] },
        "trivia": { "nodes": [] }
    }
}
```

## Reference

```sh
Usage: mago ast [OPTIONS] <FILE>
```

| Argument | Description |
| :--- | :--- |
| `<FILE>` | The PHP file to inspect. Required. |

| Flag | Description |
| :--- | :--- |
| `--tokens` | Print the lexer's token stream instead of the parsed AST. |
| `--json` | Print the output (AST or token stream) as pretty-printed JSON. |
| `--names` | Run the name resolver on the parsed AST and print fully qualified names for every symbol. Cannot be combined with `--tokens`. |
| `-h`, `--help` | Print help and exit. |

Global flags must come before `ast`. See the [CLI overview](/fundamentals/command-line-interface/) for the full list.

## Driving the parser from Rust

If you are building a tool in Rust and need a fast PHP parser, you can use Mago's crates directly:

- [`mago-syntax`](https://crates.io/crates/mago-syntax): the lexer, parser, AST node definitions, and helpers for walking the tree.
- [`mago-names`](https://crates.io/crates/mago-names): name resolution, turning a local class name into its fully qualified form.
