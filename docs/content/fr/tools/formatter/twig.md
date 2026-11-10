+++
title = "Formateur Twig"
description = "Formater la syntaxe Twig sans reformater le HTML."
nav_order = 35
nav_section = "Tools"
nav_subsection = "Twig Formatter"
+++
# Formateur Twig

`mago twig:fmt` formate les templates Twig. La recherche dans les dossiers sélectionne les fichiers `.twig` sous `source.paths`, dont `index.html.twig`, sans tenir compte de `source.extensions`. Les fichiers explicites utilisent Twig, même sans extension ou avec `.php`. `mago fmt`, `mago fix` et la commande de liste des fichiers gardent leur comportement PHP.

```sh
mago twig:fmt templates/
mago twig:fmt --check
mago twig:fmt --dry-run
mago twig:fmt --staged
mago twig:fmt --stdin-input --stdin-filepath templates/index.html.twig < templates/index.html.twig
```

La commande choisit le langage. L'entrée standard utilise toujours Twig, avec ou sans `--stdin-filepath` ; ce chemin ne sert qu'aux exclusions et aux messages. La sortie va sur stdout, sans modifier le fichier du tampon. `--staged` sélectionne les fichiers dont l'extension est exactement `.twig` et ne modifie que l'index Git. Les fichiers `.TWIG` explicites restent valides.

## Périmètre

Mago formate les balises, expressions, commentaires et indentations Twig. Les espaces suivent la [norme Twig](https://twig.symfony.com/doc/3.x/coding_standards.html). Il garde le séparateur des arguments nommés (`name: value` ou `name = value`) pour ne pas imposer une version plus récente de Twig. Les valeurs par défaut des macros gardent la forme `name = value`.

Mago n'analyse ni ne reformate le HTML, le CSS, le JavaScript ou le texte du template. Il garde le contenu des chaînes et leurs interpolations, les marqueurs de contrôle des espaces (`-` et `~`) et le corps des blocs `verbatim`. Il laisse les balises d'extension inconnues telles quelles. Il garde aussi la mise en forme d'une balise si un changement d'espacement transforme un nom, comme `not`, en opérateur. Une erreur de syntaxe empêche toute modification du fichier concerné.

Les fichiers qui utilisent l'ancien bloc `raw` restent aussi inchangés.

Mago utilise les opérateurs intégrés de Twig. Utilisez les marqueurs d'exclusion dans les fichiers qui dépendent d'opérateurs ajoutés par une extension.

Mago n'ajoute pas de saut de ligne final et ne change pas les fins de ligne du texte. `end-of-line` ne concerne que les sauts de ligne ajoutés dans les balises Twig.

Le formatage change ce que la fonction Twig `source()` renvoie pour un fichier formaté. Utilisez `@mago-format-ignore` dans les fichiers dont le texte source doit rester inchangé.

Pour limiter l'usage de la pile, le formatage limite l'imbrication du parseur à 32 niveaux. Les fichiers qui dépassent cette limite renvoient une erreur et restent inchangés.

## Indentation

Twig rend les espaces hors de ses balises comme du texte. Mago garde chaque octet de ce texte : espaces en début de ligne, lignes vides, fins de ligne et saut de ligne final. Il indente seulement le code réparti sur plusieurs lignes à l'intérieur des balises. Ces lignes suivent l'indentation existante de la balise.

Avec `print-width = 30` :

```twig
  {{
      user.name|default(
          'A guest with a longer name'
      )
  }}
```

Aucun réglage ne réécrit les espaces hors des balises. Mago garde les balises telles quelles si elles contiennent des commentaires en ligne, des noms ambigus ou des expressions trop profondes pour un formatage sûr.

## Configuration

Les réglages Twig se trouvent sous `[twig.formatter]`. Ils n'héritent pas des réglages, exclusions ou préréglages PHP. `source.excludes` s'applique aux deux outils ; `twig.formatter.excludes` ne concerne que Twig. Utiliser `mago config --show twig` ou `mago config --show twig.formatter`, avec `--schema` pour le schéma. `extends` fusionne les réglages comme d'habitude. L'ancienne table `[formatter.twig]` n'est pas prise en charge.

```toml
[twig.formatter]
enabled = true
print-width = 120
tab-width = 4
use-tabs = false
end-of-line = "auto"
```

| Option | Défaut | Effet |
| :--- | :--- | :--- |
| `enabled` | `true` | Formate les fichiers `.twig`. Si faux, les laisse inchangés, y compris via stdin et dans l'index Git. |
| `excludes` | `[]` | Préfixes de chemin ou globs propres à Twig, selon `source.glob`. |
| `print-width` | `120` | Largeur visée pour les expressions Twig. Les longues chaînes restent intactes. |
| `tab-width` | `4` | Nombre d'espaces par niveau et largeur d'une tabulation. |
| `use-tabs` | `false` | Utilise des tabulations pour indenter. |
| `end-of-line` | `"auto"` | `auto`, `lf`, `crlf` ou `cr` pour les sauts de ligne ajoutés. `auto` reprend la première fin de ligne du texte hors des balises Twig, ou LF s'il n'y en a pas. |

Pour désactiver le formatage Twig :

```toml
[twig.formatter]
enabled = false
```

## Marqueurs d'exclusion

Les marqueurs du formateur PHP fonctionnent dans les commentaires Twig :

```twig
{# @mago-format-ignore-next #}
{{  leave_this_alone  }}

{# @mago-format-ignore-start #}
{{  leave_this_region_alone  }}
{# @mago-format-ignore-end #}
```

`@mago-format-ignore` exclut tout le fichier. `-next` exclut l'instruction Twig suivante et tout son corps si elle ouvre un bloc. `@mago-formatter-ignore` et ses suffixes fonctionnent aussi.
