+++
title = "Référence de configuration du linter"
description = "Toutes les options que Mago accepte sous [linter] et [linter.rules]."
nav_order = 60
nav_section = "Tools"
nav_subsection = "Linter"
+++
# Référence de configuration

Le linter est configuré sous deux tables dans `mago.toml` : `[linter]` pour les paramètres au niveau de l'outil et `[linter.rules]` pour les paramètres par règle.

```toml
[linter]
integrations = ["symfony", "phpunit"]
excludes = ["src/Generated/"]
baseline = "linter-baseline.toml"

[linter.rules]
# Disable a rule completely
ambiguous-function-call = { enabled = false }

# Change a rule's severity level
no-else-clause = { level = "warning" }

# Configure a rule's specific options
cyclomatic-complexity = { threshold = 20 }

# Exclude specific paths from one rule
prefer-static-closure = { exclude = ["tests/"] }
```

## `[linter]`

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `excludes` | liste de chaînes | `[]` | Chemins ou globs que le linter ignore. S'ajoute à `source.excludes` global. |
| `integrations` | liste de chaînes | `[]` | Intégrations de frameworks à activer. La liste complète est sur la [page des intégrations](/tools/linter/integrations/). |
| `baseline` | chaîne | aucun | Chemin vers un fichier de baseline. Lorsqu'il est défini, le linter l'utilise comme baseline par défaut, vous n'avez donc pas à passer `--baseline` à chaque exécution. La CLI `--baseline` le remplace. |
| `baseline-variant` | chaîne | `"loose"` | Variante pour les baselines nouvellement générées. Soit `"loose"` (basée sur le compte, résiliente), soit `"strict"` (ligne exacte). Voir [variantes de baseline](/fundamentals/baseline/#two-variants). |
| `minimum-fail-level` | chaîne | `"error"` | Sévérité la plus basse qui déclenche un code de sortie non nul. Valeurs : `"note"`, `"help"`, `"warning"`, `"error"`. La CLI `--minimum-fail-level` la remplace. |

`excludes` ici s'ajoute à la liste globale. Les fichiers correspondant globalement sont toujours exclus ; cette option vous permet d'exclure des fichiers supplémentaires uniquement du linter.

```toml
[source]
excludes = ["cache/**"]              # excluded from every tool

[linter]
excludes = ["database/migrations/**"]  # additionally excluded from the linter only
```

## `[linter.rules]`

Chaque clé sous cette table est un code de règle, écrit en `kebab-case`. Chaque règle accepte les options communes ci-dessous ; certaines règles acceptent également les leurs.

### Options communes

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | booléen | variable | Active ou désactive la règle. |
| `level` | chaîne | variable | Sévérité. Valeurs : `"error"`, `"warning"`, `"help"`, `"note"`. |
| `exclude` | liste de chaînes | `[]` | Chemins ou globs que la règle ignore. Les autres règles s'appliquent toujours à ces fichiers. |

### Exclusions par règle

`exclude` est utile lorsqu'une règle est généralement précieuse mais inappropriée pour une partie de la base de code, comme du code généré ou des fixtures de tests.

```toml
[linter.rules]
prefer-static-closure = { enabled = true, exclude = ["tests/"] }
no-goto              = { exclude = ["src/Legacy/"] }
no-eval              = { exclude = ["src/Templating/Compiler.php"] }
no-global            = { exclude = ["**/*Test.php"] }
```

Chaque entrée peut être un chemin simple ou un glob :

- Les chemins simples (`"tests"`, `"tests/"`, `"src/Foo.php"`) correspondent comme préfixes par rapport au chemin de fichier relatif depuis la racine du projet.
- Les motifs glob (toute entrée contenant `*`, `?`, `[` ou `{`) correspondent au chemin relatif complet en utilisant le même moteur de glob que `source.excludes` global, avec les paramètres `[source.glob]` appliqués.

Les motifs glob dans les `exclude` par règle nécessitent Mago 1.20 ou version ultérieure. Les versions antérieures n'acceptent que les chemins préfixes simples.

`exclude` par règle n'est pas la même chose que `[linter].excludes` :

- `[linter].excludes` retire les fichiers de toutes les règles.
- L'`exclude` propre à une règle retire les fichiers de cette seule règle. Les autres règles s'appliquent toujours.

### Options spécifiques aux règles

Certaines règles acceptent des options supplémentaires. `cyclomatic-complexity` en est un exemple typique :

```toml
[linter.rules]
cyclomatic-complexity = { level = "error", threshold = 15 }
```

Pour découvrir les options pour une règle spécifique, demandez à Mago :

```sh
mago lint --explain cyclomatic-complexity
```

La référence complète par règle est sur la [page des règles](/tools/linter/rules/).
