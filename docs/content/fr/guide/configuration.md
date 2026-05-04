+++
title = "Configuration"
description = "Comment Mago découvre les fichiers de configuration, ce que fait chaque option de mago.toml, et comment partager une configuration entre projets."
nav_order = 40
nav_section = "Guide"
+++
# Configuration

Mago lit sa configuration depuis un seul fichier, généralement `mago.toml` à la racine de votre projet. Lancez `mago init` pour en générer un, ou écrivez-le à la main.

Cette page couvre la découverte de la configuration, la directive `extends`, les options globales et les sections `[source]` et `[parser]`. Les options spécifiques à chaque outil sont documentées sur la page de référence de l'outil concerné.

## Découverte

Lorsque vous ne passez pas `--config`, Mago cherche un fichier de configuration dans cet ordre :

1. Le répertoire de travail (le répertoire courant, ou le chemin donné par `--workspace`).
2. `$XDG_CONFIG_HOME` s'il est défini, par exemple `$XDG_CONFIG_HOME/mago.toml`.
3. `$HOME/.config`, par exemple `~/.config/mago.toml`.
4. `$HOME`, par exemple `~/mago.toml`.

Dans chaque emplacement, il cherche d'abord `mago.{toml,yaml,yml,json}`, puis `mago.dist.{toml,yaml,yml,json}`. La précédence des formats au sein d'un même répertoire est `toml > yaml > yml > json`. Le premier fichier trouvé l'emporte, ce qui vous permet de garder une configuration globale dans `~/.config/mago.toml` pour les projets qui n'en ont pas localement.

## Schéma pour l'éditeur

Chaque version publie un schéma JSON décrivant l'intégralité de l'arbre de configuration. Les éditeurs qui comprennent ce schéma offrent l'auto-complétion, la documentation au survol et la validation en ligne pour `mago.{toml,yaml,yml,json}`.

Le schéma est hébergé à :

- `https://mago.carthage.software/<version>/schema.json`, épinglé à une release précise comme `1.25.3`.
- `https://mago.carthage.software/latest/schema.json`, la dernière release stable.
- `https://mago.carthage.software/main/schema.json`, la build de développement depuis `main`.

Épinglez l'URL à la version de Mago installée pour que le schéma et votre binaire restent synchronisés. `mago init` écrit l'URL épinglée dans le fichier qu'il génère.

La façon de référencer le schéma dépend du format :

```toml
#:schema https://mago.carthage.software/1.25.3/schema.json
version = "1"
php-version = "8.3"
```

```yaml
# yaml-language-server: $schema=https://mago.carthage.software/1.25.3/schema.json
version: "1"
php-version: "8.3"
```

```json
{
  "$schema": "https://mago.carthage.software/1.25.3/schema.json",
  "version": "1",
  "php-version": "8.3"
}
```

Pour TOML, le commentaire est lu par le serveur de langage [Taplo](https://taplo.tamasfe.dev/), qui alimente l'extension VS Code « Even Better TOML » et le support TOML de JetBrains. Pour YAML, le commentaire est lu par le [serveur de langage YAML de Red Hat](https://github.com/redhat-developer/yaml-language-server). Pour JSON, tout éditeur moderne lit `$schema` nativement. Mago lui-même ignore la clé `$schema` et les commentaires magiques, ils n'existent que pour l'outillage des éditeurs.

Si vous régénérez le schéma en CI (par exemple, pour valider des fichiers de configuration de manière programmatique), `mago config --schema` l'imprime sur stdout.

## Partager une configuration avec `extends`

> Disponible depuis Mago 1.25. Les versions antérieures ignorent silencieusement la directive.

La directive `extends` permet à une configuration de s'appuyer sur d'autres sans copier-coller. Pratique quand plusieurs projets partagent un standard commun.

```toml
# Parent unique
extends = "vendor/some-org/mago-config/mago.toml"

# Ou une liste, appliquée de gauche à droite ; chaque couche postérieure remplace les précédentes
extends = [
  "vendor/some-org/mago-config",     # répertoire : mago.{toml,yaml,yml,json} à l'intérieur
  "configs/strict.json",              # mélanger les formats est autorisé
  "../shared/team-defaults.toml",
]

# Les clés propres à ce fichier remplacent tout ce qui vient des couches précédentes
php-version = "8.3"
```

### Résolution

Les chemins absolus sont utilisés tels quels. Les chemins relatifs sont résolus par rapport au répertoire du fichier qui déclare `extends`, et non par rapport au répertoire de travail courant. Ainsi, `mago --config some/dir/config.toml` avec `extends = "base.toml"` cherche `some/dir/base.toml`.

Les entrées de fichier doivent exister et utiliser une extension reconnue (`.toml`, `.yaml`, `.yml`, `.json`). Les entrées de répertoire sont parcourues à la recherche de `mago.{toml,yaml,yml,json}` selon cette précédence ; un répertoire sans fichier reconnu est ignoré avec un avertissement plutôt que de faire échouer la construction.

### Précédence effective

Les couches sont fusionnées en partant de la plus profonde, et la dernière l'emporte :

1. Valeurs par défaut intégrées.
2. Chaque couche `extends`, récursivement. Le `extends` propre à un parent est résolu avant que ses clés ne s'appliquent.
3. Les clés du fichier hôte.
4. Les variables d'environnement `MAGO_*` pour les [scalaires pris en charge](/guide/environment-variables/).
5. Les drapeaux CLI tels que `--php-version`, `--threads`.

### Sémantique de fusion

Par clé de premier niveau :

- Les tables et objets sont fusionnés en profondeur. Un enfant peut surcharger une seule clé d'une table imbriquée sans redéfinir toute la table.
- Les tableaux comme `source.excludes` et les listes `exclude` par règle sont concaténés, parent en premier. Si une configuration de base exclut `vendor/`, vous gardez cette exclusion et ajoutez la vôtre.
- Les scalaires (chaînes, nombres, booléens) sont écrasés par l'enfant.

```toml
# base.toml
threads = 4
[source]
excludes = ["vendor", "node_modules"]
```

```toml
# mago.toml du projet
extends = "base.toml"
threads = 8
[source]
excludes = ["build"]   # concaténé -> ["vendor", "node_modules", "build"]
```

Les cycles sont détectés via le suivi des chemins canoniques et déclenchent une erreur claire au lieu de boucler indéfiniment. L'héritage en losange (A étend B et C, qui étendent tous deux D) traite D une seule fois et fonctionne sans souci. Les couches peuvent mélanger les formats librement ; chacune est analysée par son propre driver et fusionnée à un niveau de valeur générique avant que le document final ne soit validé selon le schéma.

## Options globales

Ces clés se trouvent à la racine de `mago.toml`.

```toml
version = "1"
php-version = "8.2"
threads = 8
stack-size = 8388608     # 8 MiB
editor-url = "phpstorm://open?file=%file%&line=%line%&column=%column%"
```

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `version` | string | aucun | Fixe la version de Mago contre laquelle ce projet est testé. Accepte un majeur (`"1"`), mineur (`"1.25"`) ou exact (`"1.25.2"`). Voir [épinglage de version](#version-pinning). |
| `php-version` | string | dernière stable | La version PHP que Mago doit cibler pour l'analyse syntaxique et l'analyse. `mago init` la détecte automatiquement depuis `composer.json` quand c'est possible. |
| `allow-unsupported-php-version` | boolean | `false` | Autoriser Mago à s'exécuter sur une version PHP qu'il ne prend pas officiellement en charge. Non recommandé. |
| `no-version-check` | boolean | `false` | Désactive l'avertissement émis quand le binaire installé diverge de la version épinglée. Une divergence de version majeure est toujours fatale. |
| `threads` | integer | CPU logiques | Nombre de threads pour le travail en parallèle. |
| `stack-size` | integer | 2 MiB | Taille de pile par thread en octets. Minimum 2 MiB, maximum 8 MiB. |
| `editor-url` | string | aucun | Modèle d'URL pour les chemins de fichiers cliquables dans la sortie du terminal. Voir [intégration éditeur](#editor-integration). |

### Épinglage de version

Épingler la version fait remonter rapidement les divergences entre le binaire installé et les attentes du projet, plutôt que de produire silencieusement une sortie différente.

Trois niveaux d'épinglage :

- **Épinglage majeur** (`version = "1"`) : tout `1.x.y` satisfait l'épinglage. Une montée vers `2.x` est une erreur fatale, car une nouvelle version majeure peut introduire des défauts incompatibles, des changements de schéma ou de comportement de règles. C'est ce que `mago init` écrit par défaut.
- **Épinglage mineur** (`version = "1.25"`) : tout `1.25.y` satisfait l'épinglage. Une divergence vers un mineur différent émet un avertissement ; une divergence majeure reste fatale.
- **Épinglage exact** (`version = "1.25.2"`) : toute divergence émet un avertissement ; une divergence majeure reste fatale.

L'avertissement peut être désactivé avec `--no-version-check`, la variable d'environnement `MAGO_NO_VERSION_CHECK`, ou `no-version-check = true` dans la configuration. Aucun de ces moyens n'affecte la divergence de version majeure, qui est tout l'intérêt de l'épinglage.

Pour synchroniser le binaire installé avec l'épinglage du projet :

```sh
mago self-update --to-project-version
```

Pour les épinglages exacts, cela résout directement vers ce tag de release. Pour les épinglages majeurs ou mineurs, Mago parcourt les releases GitHub récentes et installe la plus haute qui satisfait l'épinglage. Ainsi, `version = "1"` avec 2.0 déjà sortie installe quand même la dernière release 1.x sans vous tirer en avant.

`version` est actuellement optionnel. Une future version de Mago pourrait commencer à émettre un avertissement quand il est absent, afin de préparer les projets à l'éventuelle montée vers la 2.0.

## `[source]`

La section `[source]` contrôle la manière dont Mago découvre et traite les fichiers.

### Trois catégories de chemins

Mago distingue votre code, le code tiers et le code à ignorer entièrement :

- **`paths`** sont vos fichiers source. Mago les analyse, les linte et les formate.
- **`includes`** sont les dépendances (généralement `vendor`). Mago les analyse pour pouvoir résoudre les symboles et les types, mais ne les analyse, ne les linte ni ne les réécrit jamais.
- **`excludes`** sont des chemins ou des globs que Mago ignore entièrement. Ils s'appliquent à chaque outil.

Si un fichier correspond à la fois à `paths` et à `includes`, le motif le plus spécifique l'emporte. Les chemins de fichier exacts sont les plus spécifiques, puis les chemins de répertoires plus profonds, puis les moins profonds, puis les motifs glob. Quand les motifs sont également spécifiques, `includes` l'emporte, ce qui vous permet de marquer explicitement un chemin comme dépendance.

```toml
[source]
paths     = ["src", "tests"]
includes  = ["vendor"]
excludes  = ["cache/**", "build/**", "var/**"]
extensions = ["php"]
```

Les motifs glob fonctionnent dans les trois listes :

```toml
[source]
paths    = ["src/**/*.php"]
includes = ["vendor/symfony/**/*.php"]   # only Symfony from vendor
excludes = [
  "**/*_generated.php",
  "**/tests/**",
  "src/Legacy/**",
]
```

### Référence

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `paths` | liste de strings | `[]` | Répertoires ou globs pour votre code source. Si vide, l'ensemble du workspace est scanné. |
| `includes` | liste de strings | `[]` | Répertoires ou globs pour le code tiers que Mago doit analyser sans le modifier. |
| `excludes` | liste de strings | `[]` | Globs ou chemins exclus de tous les outils. |
| `extensions` | liste de strings | `["php"]` | Extensions de fichiers traitées comme du PHP. |

### Réglages des globs

`[source.glob]` ajuste la façon dont les globs correspondent. Disponible depuis 1.19.

```toml
[source.glob]
literal-separator = true     # `*` does not match `/`; use `**` for recursion
case-insensitive  = false
backslash-escape  = true     # `\` escapes special characters
empty-alternates  = false    # `{,a}` matches "" and "a" when true
```

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `case-insensitive` | bool | `false` | Faire correspondre les motifs sans tenir compte de la casse. |
| `literal-separator` | bool | `false` | Quand `true`, `*` ne correspond pas aux séparateurs de chemin. Utilisez `**` pour la correspondance récursive. |
| `backslash-escape` | bool | `true` (false sous Windows) | Indique si `\` échappe les caractères spéciaux. |
| `empty-alternates` | bool | `false` | Indique si les alternatives vides sont autorisées. |

> Les projets générés par `mago init` mettent `literal-separator = true`. Cela fait que `*` se comporte comme la plupart des utilisateurs s'y attendent, en ne correspondant qu'à un seul niveau de répertoire, comme dans `.gitignore`.

### Exclusions par outil

Chaque outil a son propre `excludes` optionnel. Ils sont additifs : un fichier est exclu s'il correspond à la liste globale ou à la liste spécifique à l'outil.

```toml
[source]
paths    = ["src", "tests"]
excludes = ["cache/**"]            # all tools

[analyzer]
excludes = ["tests/**/*.php"]      # only the analyzer

[formatter]
excludes = ["src/**/AutoGenerated/**/*.php"]

[linter]
excludes = ["database/migrations/**"]
```

Le linter prend également en charge des exclusions de chemin par règle, utiles quand vous voulez qu'une règle ignore un chemin alors que le reste continue à s'appliquer. Les motifs glob y nécessitent Mago 1.20 ou plus récent. La référence complète se trouve sur la [page de configuration du linter](/tools/linter/configuration-reference/#per-rule-excludes).

```toml
[linter.rules]
prefer-static-closure = { exclude = ["tests/"] }
no-global             = { exclude = ["**/*Test.php"] }
```

> Utilisez `mago list-files` pour vérifier quels fichiers Mago va traiter. `mago list-files --command formatter` montre ce que le formateur va toucher, `--command analyzer` montre la vue de l'analyseur, et ainsi de suite.

## `[parser]`

```toml
[parser]
enable-short-tags = false
```

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `enable-short-tags` | boolean | `true` | Indique s'il faut reconnaître la balise courte `<?` en plus de `<?php` et `<?=`. Équivalent à la directive ini PHP `short_open_tag`. |

Désactivez les balises ouvertes courtes lorsque vos fichiers `.php` contiennent des déclarations littérales `<?xml` ou des fragments de modèles qui ne sont pas du PHP. Avec `enable-short-tags = false`, des séquences telles que `<?xml version="1.0"?>` sont traitées comme du texte intégré plutôt que comme des erreurs d'analyse. Le compromis : tout code qui s'appuie sur `<?` comme balise d'ouverture PHP ne sera plus reconnu.

## Intégration éditeur

Mago peut afficher les chemins de fichiers dans la sortie de diagnostic sous forme de [liens hypertextes OSC 8](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda). Cliquez sur le chemin dans votre terminal et votre éditeur ouvre le fichier à la bonne ligne et la bonne colonne. Les terminaux pris en charge incluent iTerm2, WezTerm, Kitty, Windows Terminal, Ghostty, et quelques autres.

Mago détecte automatiquement l'éditeur en cours d'exécution quand c'est possible. Sur macOS, il lit `__CFBundleIdentifier` ; ailleurs il vérifie `TERM_PROGRAM`. Les éditeurs suivants sont reconnus directement :

- PhpStorm, IntelliJ IDEA, WebStorm
- VS Code, VS Code Insiders
- Zed
- Sublime Text

Si la détection automatique échoue, configurez l'URL explicitement. La précédence est de type premier-trouvé-l'emporte :

1. Variable d'environnement `MAGO_EDITOR_URL`.
2. `editor-url` dans `mago.toml`.
3. Détection automatique.

```sh
export MAGO_EDITOR_URL="vscode://file/%file%:%line%:%column%"
```

```toml
editor-url = "phpstorm://open?file=%file%&line=%line%&column=%column%"
```

| Variable | Signification |
| :--- | :--- |
| `%file%` | Chemin absolu vers le fichier. |
| `%line%` | Numéro de ligne, base 1. |
| `%column%` | Numéro de colonne, base 1. |

Modèles courants :

| Éditeur | Modèle |
| :--- | :--- |
| VS Code | `vscode://file/%file%:%line%:%column%` |
| VS Code Insiders | `vscode-insiders://file/%file%:%line%:%column%` |
| Cursor | `cursor://file/%file%:%line%:%column%` |
| Windsurf | `windsurf://file/%file%:%line%:%column%` |
| PhpStorm / IntelliJ | `phpstorm://open?file=%file%&line=%line%&column=%column%` |
| Zed | `zed://file/%file%:%line%:%column%` |
| Sublime Text | `subl://open?url=file://%file%&line=%line%&column=%column%` |
| Emacs | `emacs://open?url=file://%file%&line=%line%&column=%column%` |
| Atom | `atom://core/open/file?filename=%file%&line=%line%&column=%column%` |

> Les liens hypertextes ne sont rendus que lorsque la sortie est un terminal avec les couleurs activées. Ils sont automatiquement supprimés quand la sortie est redirigée vers un pipe ou que `--colors=never` est utilisé, afin de ne pas interférer avec les scripts ou la CI.

Les liens hypertextes apparaissent dans les formats de rapport `rich` (par défaut), `medium`, `short` et `emacs`. Les formats lisibles par machine (`json`, `github`, `gitlab`, `checkstyle`, `sarif`) ne sont pas affectés.

## Configuration spécifique aux outils

Chaque outil a sa propre page de référence couvrant ses options :

- [Linter](/tools/linter/configuration-reference/)
- [Formateur](/tools/formatter/configuration-reference/)
- [Analyseur](/tools/analyzer/configuration-reference/)
- [Guard](/tools/guard/configuration-reference/)

## Inspecter la configuration fusionnée

`mago config` affiche la configuration que Mago utilise réellement, après fusion des défauts, de chaque couche `extends`, des variables d'environnement et des drapeaux CLI. Pratique quand quelque chose ne se comporte pas comme prévu.

```sh
mago config                       # full config as pretty-printed JSON
mago config --show linter         # only the [linter] section
mago config --show formatter
mago config --default             # the built-in defaults
mago config --schema              # JSON Schema for the whole config
mago config --schema --show linter
```

| Drapeau | Description |
| :--- | :--- |
| `--show <SECTION>` | Affiche une seule section. Valeurs : `source`, `parser`, `linter`, `formatter`, `analyzer`, `guard`. |
| `--default` | Affiche les valeurs par défaut intégrées au lieu du résultat fusionné. |
| `--schema` | Affiche le JSON Schema, utile pour l'intégration IDE ou un outillage externe. |
| `-h`, `--help` | Affiche l'aide et quitte. |

Les drapeaux globaux doivent venir avant `config`. Voir l'[aperçu CLI](/fundamentals/command-line-interface/) pour la liste complète.
