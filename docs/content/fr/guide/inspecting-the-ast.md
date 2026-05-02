+++
title = "Inspecter l'AST"
description = "La commande mago ast pour examiner la sortie du parseur. Un outil de débogage, pas une étape du flux quotidien."
nav_order = 75
nav_section = "Guide"
+++
# Inspecter l'AST

`mago ast` affiche soit l'arbre syntaxique abstrait, soit le flux de tokens d'un seul fichier PHP. Utile pour déboguer un parsing délicat, comprendre comment Mago perçoit un bout de syntaxe, ou alimenter un autre outil qui veut le parseur de Mago. Cette commande ne fait pas partie du flux régulier formateur / linter / analyseur / guard ; traitez-la comme une commande de débogage, au même titre que `list-files` et autres.

## Vue arbre

Avec `example.php` :

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

## Vue tokens

`--tokens` affiche le flux de tokens du lexer à la place. Utile pour déboguer des problèmes de syntaxe bas niveau.

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

## Sortie JSON

`--json` bascule l'une ou l'autre vue en JSON formaté. À combiner avec `--tokens` pour le flux de tokens en JSON, ou seul pour l'AST complet.

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

## Référence

```sh
Usage: mago ast [OPTIONS] <FILE>
```

| Argument | Description |
| :--- | :--- |
| `<FILE>` | Le fichier PHP à inspecter. Requis. |

| Option | Description |
| :--- | :--- |
| `--tokens` | Affiche le flux de tokens du lexer à la place de l'AST. |
| `--json` | Affiche la sortie (AST ou flux de tokens) en JSON formaté. |
| `--names` | Exécute le résolveur de noms sur l'AST analysé et affiche les noms pleinement qualifiés de chaque symbole. Incompatible avec `--tokens`. |
| `-h`, `--help` | Affiche l'aide et quitte. |

Les options globales doivent précéder `ast`. Voir l'[aperçu CLI](/fundamentals/command-line-interface/) pour la liste complète.

## Piloter le parseur depuis Rust

Si vous construisez un outil en Rust et avez besoin d'un parseur PHP rapide, vous pouvez utiliser directement les crates de Mago :

- [`mago-syntax`](https://crates.io/crates/mago-syntax) : le lexer, le parseur, les définitions de nœuds AST et des helpers pour parcourir l'arbre.
- [`mago-names`](https://crates.io/crates/mago-names) : la résolution de noms, qui transforme un nom de classe local en sa forme pleinement qualifiée.
