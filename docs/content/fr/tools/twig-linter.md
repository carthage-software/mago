+++
title = "Linter Twig"
description = "Vérifier Twig avec les rapports, pragmas et corrections de Mago."
nav_order = 40
nav_section = "Tools"
nav_subsection = "Twig Linter"
+++
# Linter Twig

`mago twig:lint` vérifie Twig. La recherche dans les dossiers sélectionne les fichiers `.twig` sous `source.paths`, dont `index.html.twig`, sans tenir compte de `source.extensions`. Les fichiers explicites, chemins stdin et fichiers de remplacement utilisent Twig, même sans extension ou avec `.php`. La commande n'utilise jamais le parseur ou les règles PHP.

La recherche dans les dossiers et `--staged` respectent la casse de `.twig`. Un chemin `.TWIG` explicite reste valide.

```sh
mago twig:lint templates/
mago twig:lint --only twig-no-dump
mago twig:lint --list-rules --json
mago twig:lint --explain twig-no-raw
mago twig:lint --stdin-input templates/page.html.twig < templates/page.html.twig
mago twig:lint templates/ --fix
```

`--semantics` vérifie seulement la syntaxe Twig. Il ne lance ni les règles de style ni l'analyse des types. `--only` attend les codes avec le préfixe `twig-` et active ces règles, même si leur réglage les désactive. Il respecte toutefois `enabled = false`, les exclusions et la version minimale de chaque règle. `--pedantic` active toutes les règles prises en charge, y compris les choix de politique du projet.

La commande utilise les rapports, seuils de gravité, statistiques, baselines et options de correction de Mago. `--staged` sélectionne les fichiers `.twig` indexés par Git et indexe les corrections seulement si leur copie de travail correspond à l'index.

`--substitute ORIG=TEMP` exclut l'original et vérifie le remplacement comme Twig. Les deux chemins doivent être absolus. Le rapport utilise le chemin du remplacement, sans contrainte d'extension. `--stdin-input` exige un chemin pour les rapports et baselines.

La commande choisit le langage ; `--only` choisit les règles Twig. La syntaxe Twig reste vérifiée même si aucune règle ne correspond. `mago lint`, `mago analyze`, `mago fix` et `mago list-files` gardent leur comportement. Ils ne lancent pas les règles du linter Twig.

## Configuration

```toml
[twig.linter]
enabled = true
version = "3.30"
excludes = ["templates/generated/**"]

[twig.linter.rules.no-dump]
level = "error"
exclude = ["templates/debug/**"]

[twig.linter.rules.variable-name]
enabled = true

[twig.linter.rules.disallowed-functions]
enabled = true
names = ["source"]
```

| Option | Valeur par défaut | Sens |
| :--- | :--- | :--- |
| `enabled` | `true` | Lancer `twig:lint`. Si faux, ne chercher ni vérifier de fichiers. Les autres outils ne changent pas. |
| `version` | `"3.0"` | Plus ancienne version Twig 3 prise en charge, au format `3.MINOR`. La définir pour activer les règles et corrections propres à une version. Mago ne la déduit pas de Composer et ne vérifie pas toutes les fonctions du langage contre elle. Le parseur accepte la syntaxe Twig actuelle. |
| `excludes` | `[]` | Préfixes de chemins ou globs propres à Twig. Exclure avant l'analyse syntaxique. |
| `baseline` | Aucune | Chemin relatif à `source.workspace` ou absolu. `--baseline` le remplace. |
| `baseline-variant` | `"loose"` | Écrire une baseline `loose` par nombre ou `strict` par ligne. |
| `minimum-fail-level` | `"error"` | Niveau minimal qui fait échouer la commande. `--minimum-fail-level` le remplace. |
| `rules.<nom>.enabled` | Selon la règle | Activer ou désactiver la règle. Omettre `twig-` dans la clé de configuration. |
| `rules.<nom>.level` | Selon la règle | `note`, `help`, `warning` ou `error`. `twig.linter.minimum-fail-level` fixe le seuil d'échec de la commande. |
| `rules.<nom>.exclude` | `[]` | Exclusions d'une règle, selon `source.glob`, sur les noms relatifs à l'espace de travail. |
| `rules.disallowed-functions.names` | `[]` | Noms exacts des fonctions interdites, y compris les alias de macros. Activer aussi la règle. |
| `rules.disallowed-filters.names` | `[]` | Noms exacts des filtres interdits. Activer aussi la règle. |
| `rules.disallowed-tags.names` | `[]` | Noms exacts des balises interdites, y compris les balises d'extension. Activer aussi la règle. |

`source.paths`, `source.excludes` et `source.glob` sont communs aux outils. Les règles, exclusions, baselines et seuils PHP de `[linter]` n'affectent pas `twig:lint`. `extends` fusionne `[twig.linter]` comme d'habitude. Les champs et noms de règles inconnus sont des erreurs. Consulter `mago config --show twig.linter` et `mago config --schema --show twig.linter`. `config --show linter` reste propre à PHP.

`mago config --show twig` affiche les outils Twig configurés. Un outil omis prend ses propres valeurs par défaut à son lancement ; `mago config --show twig.linter --default` affiche celles du linter.

## Règles

La [page des règles](/tools/twig-linter/rules/) liste chaque règle avec son niveau par défaut, la version de Twig requise, ses réglages et ses exemples. Les codes des règles commencent par `twig-` ; les clés de configuration omettent ce préfixe. Une règle désactivée par défaut est une politique facultative, pas une erreur de syntaxe Twig.

Les vérifications de fonctions natives respectent les alias de macros et leur portée lexicale. Les noms d'extension inconnus restent valides. Les interdictions configurées s'appliquent aussi aux noms de macros importées. Les règles de nommage laissent les noms non ASCII intacts. Les clés dynamiques, conversions unaires de chaînes et conversions numériques hors plage restent inconnues.

Les références sont la [norme Twig officielle](https://twig.symfony.com/doc/3.x/coding_standards.html), le [catalogue Twig-CS-Fixer](https://twigcsfixer.github.io/rules) et le [code source Twig](https://github.com/twigphp/Twig). Le [formateur Twig](/tools/formatter/twig/) garde la charge des espaces, retraits, virgules et autres choix de mise en page. Mago ne reprend pas les corrections de texte ou de fin de fichier qui peuvent changer le rendu.

## Pragmas

Seuls des espaces et d'autres commentaires peuvent séparer un pragma `{# #}` sur sa propre ligne de son instruction. Du texte rendu ou une balise de fermeture arrête cette portée ; le pragma ne masque pas une instruction plus loin.

Un pragma `#` sur sa propre ligne dans une balise de code ne couvre que la suite de cette balise. Il ne s'étend ni au corps d'un `if`/bloc, ni à une balise plus loin.

```twig
{# @mago-expect lint:twig-no-dump(2) Sortie de débogage voulue dans cette branche. #}
{% if debug %}
    {{ dump(first) }}
    {{ dump(second) }}
{% endif %}
```

Les catégories `lint` et `linter`, listes séparées par des virgules et préfixes répétés sont valides. Un commentaire sur sa propre ligne avant une instruction couvre cette instruction et son corps. Un commentaire en fin de ligne couvre sa ligne. Les commentaires imbriqués ont priorité. `(N)` limite le nombre de correspondances ; sans nombre, un pragma de portée garde le comportement usuel sans limite dans cette portée. `--only` ne signale pas les attentes liées aux codes inactifs. Lors d'un passage complet, les règles désactivées ou exclues peuvent laisser des attentes non remplies, comme en PHP.

Mago signale les ignores inutilisés et attentes non remplies. Leur nettoyage sûr reste dans le corps du commentaire, conserve délimiteurs et marqueurs de suppression d'espaces, et garde les nombres `(1)` explicites. Un nettoyage groupé peut demander un second passage de `twig:lint --fix`. `mago fix` ne lance pas les règles du linter Twig.

## Sûreté des corrections et limites

Les corrections sûres ne changent que séparateurs nommés, noms de fermeture manquants, guillemets de chaînes simples ou corps de pragmas. Elles ne réécrivent ni espaces rendus, ni contenu des chaînes, ni ordre des expressions. La correction des séparateurs nommés exige une version minimale explicite de 3.12 ou plus. `--potentially-unsafe` et `--unsafe` n'ajoutent aucune correction Twig pour le moment.

`twig:lint` refuse `--format-after-fix` avant toute lecture ou écriture de fichiers. Lancer le formatage Twig dans une étape distincte ; cette commande ne change pas le choix de langage du formateur actuel.

Les fichiers avec des blocs `raw` anciens gardent leurs diagnostics, mais Mago ne les modifie à aucun niveau de sûreté, pas même leurs pragmas. Le traitement ancien ou personnalisé de `raw` peut changer la façon dont Twig en lit les limites. Le mode `--semantics` ne lit pas les pragmas du linter.

Mago suppose les délimiteurs, interpolations et opérateurs natifs de Twig. Exclure les fichiers ou règles concernés si un lexer personnalisé en change le sens. Les corps des balises inconnues et de `verbatim` restent opaques. Mago ne connaît pas les remplacements de fonctions/filtres natifs ni les parseurs d'appels personnalisés ; régler les règles concernées selon le projet.

Toute modification du code source change la valeur renvoyée par `source()`. Exclure les fichiers dont le contrat porte sur les octets du source.

Le parseur a un budget de 32 niveaux ; une construction peut en prendre plusieurs. `twig-depth-limit` indique que Mago n'a pas vérifié le template, pas que Twig le refuse. Les autres erreurs portent le code `twig-syntax-error`, et aucune règle ne s'exécute sur un tel template. Les pragmas ne peuvent masquer aucun de ces deux codes. Les longues chaînes d'expressions plates restent vérifiées. Les tests de la bibliothèque en mode debug passent avec une pile de 4 Mio ; des piles plus petites peuvent ne pas suffire. Mago n'augmente pas la pile des workers pour masquer ces échecs.
