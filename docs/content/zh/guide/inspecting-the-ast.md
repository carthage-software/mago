+++
title = "查看 AST"
description = "mago ast 命令用于查看解析器输出。这是调试工具,不是日常流程的一部分。"
nav_order = 75
nav_section = "指南"
+++
# 查看 AST

`mago ast` 打印一个 PHP 文件的抽象语法树或 token 流。适合用来调试棘手的解析、了解 Mago 如何看待某段语法,或者把输出提供给其他需要 Mago 解析器的工具。它并不属于格式化器 / linter / 分析器 / guard 的常规流程;请把它当作调试命令,和 `list-files` 之类的命令一起对待。

## 树状视图

给定 `example.php`:

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

## Token 视图

`--tokens` 改为打印词法分析器的 token 流。可用于排查低层语法问题。

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

## JSON 输出

`--json` 把上述任一视图切换为美化后的 JSON。可与 `--tokens` 组合得到 token 流 JSON,或单独使用得到完整 AST。

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

## 参考

```sh
Usage: mago ast [OPTIONS] <FILE>
```

| 参数 | 说明 |
| :--- | :--- |
| `<FILE>` | 要检查的 PHP 文件。必填。 |

| 选项 | 说明 |
| :--- | :--- |
| `--tokens` | 打印词法分析器的 token 流而非解析后的 AST。 |
| `--json` | 以美化后的 JSON 输出(AST 或 token 流)。 |
| `--names` | 在解析后的 AST 上运行名称解析器,打印每个符号的完全限定名。不能与 `--tokens` 同时使用。 |
| `-h`, `--help` | 打印帮助并退出。 |

全局选项必须出现在 `ast` 之前。完整列表见 [CLI 概览](/fundamentals/command-line-interface/)。

## 在 Rust 中直接驱动解析器

如果你正用 Rust 构建工具并需要一个快速的 PHP 解析器,可以直接使用 Mago 的 crate:

- [`mago-syntax`](https://crates.io/crates/mago-syntax):词法分析器、语法分析器、AST 节点定义,以及遍历 AST 的辅助工具。
- [`mago-names`](https://crates.io/crates/mago-names):名称解析,将本地类名转换为其完全限定形式。
