+++
title = "Référence de configuration de l'analyseur"
description = "Toutes les options que Mago accepte sous [analyzer]."
nav_order = 30
nav_section = "Tools"
nav_subsection = "Analyzer"
+++
# Référence de configuration

Les paramètres se trouvent sous `[analyzer]` dans `mago.toml`.

```toml
[analyzer]
ignore = ["mixed-argument"]
baseline = "analyzer-baseline.toml"
```

## Options générales

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `excludes` | `string[]` | `[]` | Chemins ou motifs glob à exclure de l'analyse. S'ajoute à `[source].excludes`. |
| `ignore` | `(string \| object)[]` | `[]` | Codes de problèmes à ignorer, optionnellement délimités à des chemins spécifiques. Voir ci-dessous. |
| `baseline` | `string` | non défini | Chemin vers un fichier de baseline. Équivalent à passer `--baseline` à chaque exécution. L'indicateur CLI le remplace. |
| `baseline-variant` | `string` | `"loose"` | Format pour les baselines nouvellement générées. `"loose"` (basée sur le compte) ou `"strict"` (correspondance exacte de ligne). Voir [baseline](/fundamentals/baseline/). |
| `minimum-fail-level` | `string` | `"error"` | Sévérité minimale qui provoque un code de sortie non nul. L'un de `"note"`, `"help"`, `"warning"`, `"error"`. Remplacé par `--minimum-fail-level`. |

`excludes` ici s'ajoute à ce que vous avez défini dans `[source].excludes` ; il ne réduit jamais la liste globale.

```toml
[source]
excludes = ["cache/**"]

[analyzer]
excludes = ["tests/**/*.php"]
```

### Ignorer par chemin

`ignore` accepte des chaînes simples, des objets à un seul chemin et des objets à plusieurs chemins, mélangés librement :

```toml
[analyzer]
ignore = [
    "mixed-argument",
    { code = "missing-return-type", in = "tests/" },
    { code = "unused-parameter", in = ["tests/", "src/Generated/"] },
]
```

Chaque entrée dans `in` est soit un préfixe de répertoire ou de fichier, soit un motif glob. Toute valeur contenant `*`, `?`, `[` ou `{` est traitée comme un glob et associée au chemin relatif complet ; tout le reste est associé comme préfixe. `"tests"` et `"tests/"` correspondent tous deux à chaque fichier sous `tests`.

```toml
[analyzer]
ignore = [
    { code = "mixed-assignment", in = [
        "tests/",
        "src/Legacy/**/*.php",
        "modules/*/Generated/*.php",
    ] },
]
```

La correspondance glob respecte les paramètres globaux du projet sous `[source.glob]`, donc des bascules comme `literal-separator` et `case-insensitive` s'appliquent ici aussi.

`excludes` et `ignore` ne sont pas la même chose. `excludes` retire entièrement les fichiers de l'analyse, ils ne sont donc pas analysés pour les informations de type. `ignore` analyse toujours le fichier mais supprime les codes listés dans la sortie.

## Indicateurs de fonctionnalité

Ces indicateurs activent ou désactivent des analyses individuelles. Les valeurs par défaut sont ajustées pour un usage quotidien ; activez-les à mesure que votre base de code se renforce.

| Option | Défaut | Description |
| :--- | :--- | :--- |
| `find-unused-expressions` | `true` | Signale les expressions dont le résultat est ignoré, comme `$a + $b;`. |
| `find-unused-definitions` | `true` | Signale les définitions privées qui ne sont jamais référencées. |
| `find-overly-wide-return-types` | `false` | Avertit lorsqu'un type de retour déclaré contient une branche que le corps ne produit jamais, comme `: string\|false` sur une fonction qui retourne toujours une chaîne. Disponible depuis 1.20.0. |
| `analyze-dead-code` | `false` | Analyse le code qui semble inaccessible. |
| `memoize-properties` | `true` | Suit les valeurs littérales de propriété pour une inférence plus précise, au prix d'un peu de mémoire. |
| `allow-possibly-undefined-array-keys` | `true` | Autorise l'accès aux clés qui peuvent manquer sans le signaler. |
| `check-throws` | `false` | Signale les exceptions non capturées et non déclarées avec `@throws`. |
| `check-missing-override` | `false` | Signale les attributs `#[Override]` manquants sur les méthodes redéfinissantes (PHP 8.3+). |
| `find-unused-parameters` | `false` | Signale les paramètres qui ne sont jamais lus. |
| `strict-list-index-checks` | `false` | Exige que tout entier utilisé comme index de liste soit prouvé non négatif. |
| `no-boolean-literal-comparison` | `false` | Interdit les comparaisons directes aux littéraux booléens comme `$a === true`. |
| `check-missing-type-hints` | `false` | Signale les indications de types manquantes sur les paramètres, propriétés et types de retour. |
| `check-closure-missing-type-hints` | `false` | Étend la vérification d'indications de types aux closures (nécessite `check-missing-type-hints`). |
| `check-arrow-function-missing-type-hints` | `false` | Étend la vérification d'indications de types aux fonctions fléchées (nécessite `check-missing-type-hints`). |
| `allow-implicit-pipe-callable-types` | `false` | Ignore les vérifications d'indications de types des closures / fonctions fléchées lorsque le callable est l'opérande de droite de `\|>`. |
| `register-super-globals` | `true` | Enregistre automatiquement les superglobales PHP comme `$_GET` et `$_POST`. |
| `trust-existence-checks` | `true` | Affine les types selon `method_exists()`, `property_exists()`, `function_exists()` et `defined()`. |
| `check-property-initialization` | `false` | Vérifie que les propriétés typées sont initialisées dans un constructeur ou un initialiseur de classe. |
| `check-use-statements` | `false` | Signale les instructions use qui importent des classes, fonctions ou constantes inexistantes. |
| `check-name-casing` | `false` | Signale les casses incorrectes lors du référencement des classes, fonctions, etc. Aide à prévenir les échecs d'autoload sur les systèmes de fichiers sensibles à la casse. |
| `enforce-class-finality` | `false` | Signale les classes qui ne sont pas `final`, `abstract` ou annotées `@api` et n'ont pas d'enfants. |
| `require-api-or-internal` | `false` | Exige que les classes abstraites, interfaces et traits soient annotés `@api` ou `@internal`. |
| `check-experimental` | `false` | Signale l'utilisation de symboles `@experimental` depuis des contextes non expérimentaux. Disponible depuis 1.19.0. |
| `allow-side-effects-in-conditions` | `true` | Lorsque `false`, signale les appels à des fonctions impures à l'intérieur des conditions de `if`, `while`, `for`, ternaire ou `match`. |

## Initialisation des propriétés

Lorsque `check-property-initialization` est activé, l'analyseur signale deux problèmes :

- `missing-constructor` pour les classes avec des propriétés typées et sans constructeur.
- `uninitialized-property` pour les propriétés typées non assignées dans le constructeur.

`class-initializers` vous permet de marquer des méthodes supplémentaires qui doivent compter comme initialiseurs, aux côtés de `__construct`. Les propriétés assignées dans ces méthodes sont traitées comme définitivement initialisées. C'est utile pour les frameworks qui utilisent des méthodes de cycle de vie.

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `class-initializers` | `string[]` | `[]` | Noms de méthodes traitées comme initialiseurs de classe. |

```toml
[analyzer]
check-property-initialization = true
class-initializers = ["setUp", "initialize", "boot"]
```

Avec cette configuration, le code suivant ne déclenche pas un faux positif :

```php
class MyTest extends TestCase
{
    private string $name;

    protected function setUp(): void
    {
        $this->name = "test";
    }
}
```

## Filtrage d'exceptions

Lorsque `check-throws` est activé, deux options vous permettent d'ignorer des exceptions spécifiques.

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `unchecked-exceptions` | `string[]` | `[]` | Exceptions à ignorer, y compris toutes les sous-classes (conscient de la hiérarchie). |
| `unchecked-exception-classes` | `string[]` | `[]` | Exceptions à ignorer comme correspondances exactes de classe uniquement. Les sous-classes et parents sont toujours vérifiés. |

```toml
[analyzer]
check-throws = true

unchecked-exceptions = [
    "LogicException",
    "Psl\\Type\\Exception\\ExceptionInterface",
]

unchecked-exception-classes = [
    "Psl\\File\\Exception\\FileNotFoundException",
]
```

Utilisez `unchecked-exceptions` pour faire taire toute une catégorie, comme chaque sous-classe de `LogicException`. Utilisez `unchecked-exception-classes` lorsque vous voulez ignorer une exception spécifique tout en suivant les frères et parents.

## Détection d'API expérimentale

Définissez `check-experimental = true` pour signaler l'utilisation de symboles `@experimental` depuis du code non expérimental. Marquez le symbole avec la balise PHPDoc :

```php
/** @experimental */
class UnstableApi {}

/** @experimental */
function beta_feature(): void {}
```

L'analyseur avertit lorsque ceux-ci sont utilisés depuis du code stable :

```php
new UnstableApi();              // warning
beta_feature();                 // warning
class MyService extends UnstableApi {}  // warning
```

L'utilisation depuis un autre contexte expérimental est autorisée :

```php
/** @experimental */
function also_experimental(): void {
    new UnstableApi();
    beta_feature();
}

class StableService {
    /** @experimental */
    public function experimentalMethod(): void {
        new UnstableApi();
    }
}
```

## Plugins

Les plugins fournissent des fournisseurs de types pour les bibliothèques et frameworks, afin que les fonctions retournent des types précis au lieu de génériques.

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `disable-default-plugins` | `bool` | `false` | Désactive tous les plugins par défaut. Seuls les noms que vous listez dans `plugins` sont actifs. |
| `plugins` | `string[]` | `[]` | Plugins à activer, par nom ou alias. |

### Plugins disponibles

| Plugin | Alias | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `stdlib` | `standard`, `std`, `php-stdlib` | activé | Fonctions intégrées PHP : `strlen`, `array_*`, `json_*`, et autres. |
| `psl` | `php-standard-library`, `azjezz-psl` | désactivé | [php-standard-library](https://github.com/php-standard-library/php-standard-library). |
| `flow-php` | `flow`, `flow-etl` | désactivé | [flow-php/etl](https://github.com/flow-php/etl). |
| `psr-container` | `psr-11` | désactivé | [psr/container](https://github.com/php-fig/container). |

Par exemple, le plugin `stdlib` apprend à l'analyseur que `strlen($s)` retourne `int<0, max>`, que `json_decode($json, true)` retourne `array<string, mixed>` et que `array_filter($array)` conserve la forme d'entrée mais peut perdre des éléments.

### Exemples

Utiliser les valeurs par défaut (juste `stdlib`) :

```toml
[analyzer]
```

Activer des plugins supplémentaires :

```toml
[analyzer]
plugins = ["psl", "flow-php", "psr-container"]
```

Tout désactiver :

```toml
[analyzer]
disable-default-plugins = true
```

Utiliser un seul plugin :

```toml
[analyzer]
disable-default-plugins = true
plugins = ["psl"]
```

Les alias de plugin fonctionnent partout, donc `plugins = ["std"]` est identique à `plugins = ["stdlib"]`.

## Mode strict

L'analyseur s'exécute à une rigueur modérée par défaut. Augmentez-la en activant plus de vérifications ; assouplissez-la pour le code hérité.

### Rigueur maximale

```toml
[analyzer]
find-unused-expressions = true
find-unused-definitions = true
find-overly-wide-return-types = true
analyze-dead-code = true
check-throws = true
check-missing-override = true
find-unused-parameters = true
check-missing-type-hints = true
check-closure-missing-type-hints = true
check-arrow-function-missing-type-hints = true
enforce-class-finality = true
require-api-or-internal = true
check-experimental = true

strict-list-index-checks = true
no-boolean-literal-comparison = true

allow-possibly-undefined-array-keys = false
trust-existence-checks = false
```

### Mode indulgent

```toml
[analyzer]
check-missing-type-hints = false
strict-list-index-checks = false
no-boolean-literal-comparison = false
enforce-class-finality = false
require-api-or-internal = false

allow-possibly-undefined-array-keys = true
trust-existence-checks = true

check-throws = false
```

Lors de l'introduction de Mago dans une base de code existante, commencez en mode indulgent avec une [baseline](/fundamentals/baseline/) et serrez les vis à mesure que le code s'améliore.

### Notes sur les indicateurs individuels

`trust-existence-checks` décide si l'analyseur affine sur les vérifications à l'exécution. Avec lui activé (le défaut), ceci convient :

```php
function process(object $obj): mixed
{
    if (method_exists($obj, 'toArray')) {
        return $obj->toArray();
    }

    return null;
}
```

Désactivez-le et l'appel exige une garantie de type explicite à la place.

`allow-implicit-pipe-callable-types` ignore les vérifications d'indications de types des closures / fonctions fléchées lorsque le callable est l'opérande de droite de `|>`. L'opérande de gauche du pipe porte assez d'informations de type pour dériver le paramètre, donc l'indication manquante est inoffensive là.

## Réglage des performances

L'analyseur utilise des seuils internes pour équilibrer profondeur et vitesse. Les paramètres se trouvent sous `[analyzer.performance]`.

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `saturation-complexity-threshold` | `u16` | `8192` | Nombre maximal de clauses pendant la saturation CNF. |
| `disjunction-complexity-threshold` | `u16` | `4096` | Nombre maximal de clauses par côté dans les opérations OR. |
| `negation-complexity-threshold` | `u16` | `4096` | Complexité cumulative maximale lors de la négation des formules. |
| `consensus-limit-threshold` | `u16` | `256` | Limite supérieure pour les passes d'optimisation par consensus. |
| `formula-size-threshold` | `u16` | `512` | Taille maximale de la formule logique avant que la simplification soit ignorée. |
| `string-combination-threshold` | `u16` | `128` | Nombre maximal de chaînes littérales suivies avant la généralisation à `string`. |
| `integer-combination-threshold` | `u16` | `128` | Nombre maximal d'entiers littéraux suivis avant la généralisation à `int`. |
| `array-combination-threshold` | `u16` | `32` | Nombre maximal de formes de tableaux clés scellés suivies individuellement avant fusion. |
| `loop-assignment-depth-threshold` | `u8` | `1` | Profondeur maximale d'itération à point fixe de boucle. `0` désactive la ré-itération. |

`string-concat-combination-threshold` est toujours accepté comme alias pour `string-combination-threshold`.

### Quand ajuster

Les valeurs par défaut fonctionnent pour la plupart des projets. Abaissez les seuils si l'analyse semble trop lente au prix d'une certaine précision d'inférence. Augmentez-les si vous avez besoin d'une inférence plus profonde sur du code hautement conditionnel.

### Ce que chaque seuil contrôle

L'analyseur transforme les contraintes de type en formules logiques sous Forme Normale Conjonctive (CNF). Celles-ci peuvent croître exponentiellement avec des conditions complexes, donc les seuils empêchent un calcul incontrôlé.

- La complexité de saturation plafonne les clauses traitées pendant la simplification de formule. Une fois dépassée, la simplification s'arrête tôt.
- La complexité de disjonction borne la croissance des clauses lors de la combinaison `OR`. Les unions larges et de nombreuses branches peuvent atteindre cela.
- La complexité de négation borne l'expansion lors de la négation des formules, par exemple le calcul des branches `else` à partir d'un `if` compliqué.
- La limite de consensus plafonne une passe d'optimisation qui détecte les tautologies logiques. Des valeurs plus élevées peuvent trouver plus de simplifications.
- La taille de formule est le plafond global de complexité avant que l'analyseur ne se rabatte sur une inférence plus simple.
- Les seuils de combinaison de chaînes / entiers plafonnent combien de valeurs littérales sont suivies avant que l'analyseur ne s'élargisse à `string` ou `int`. Sans ceux-ci, de très grands tableaux ou instructions switch pousseraient le coût de combinaison à O(n²).
- Le seuil de combinaison de tableaux plafonne combien de formes distinctes de tableaux clés scellés sont gardées séparées pendant la combinaison de types. Lorsque du code procédural accumule de nombreuses formes légèrement différentes pour la même variable à travers les branches, le combinateur garde chacune jusqu'à ce que ce seuil soit atteint et les fusionne en une forme généralisée. Augmentez-le pour du code qui dépend d'un raffinement très précis par clé.
- Le seuil de profondeur d'affectation de boucle plafonne combien d'itérations à point fixe l'analyseur de boucle exécute sur chaque corps de boucle. Avec une chaîne de `N` dépendances portées par la boucle, jusqu'à `N` passes supplémentaires peuvent être nécessaires pour que les types à la fin de la chaîne se stabilisent complètement ; chaque passe ré-analyse tout le corps. Le défaut de `1` est suffisant pour presque tout code réel. Augmentez-le à `2` ou `3` pour les bases de code qui ont besoin d'un raffinement très précis de chaînes profondes portées par les boucles. `0` désactive complètement la ré-itération, ce qui est le réglage le plus rapide mais peut laisser certains types auto-dépendants plus larges que nécessaire.

### Exemples

Analyse rapide, précision moindre :

```toml
[analyzer.performance]
saturation-complexity-threshold = 2048
disjunction-complexity-threshold = 1024
negation-complexity-threshold = 1024
consensus-limit-threshold = 64
formula-size-threshold = 128
string-combination-threshold = 64
integer-combination-threshold = 64
array-combination-threshold = 16
loop-assignment-depth-threshold = 1
```

Analyse profonde, plus lente :

```toml
[analyzer.performance]
saturation-complexity-threshold = 16384
disjunction-complexity-threshold = 8192
negation-complexity-threshold = 8192
consensus-limit-threshold = 512
formula-size-threshold = 1024
string-combination-threshold = 256
integer-combination-threshold = 256
array-combination-threshold = 256
loop-assignment-depth-threshold = 4
```

Augmenter les seuils peut faire varier sensiblement le temps d'analyse sur les bases de code avec une lourde logique conditionnelle ou des fichiers contenant des milliers d'opérations sur tableau. Testez sur votre projet avant de déployer en CI.

### Diagnostiquer les exécutions lentes

Si Mago semble lent, cela pointe généralement vers un bug dans Mago plutôt que vers un comportement normal. À titre de référence, sur un Apple M1 Pro l'analyseur couvre tout [`WordPress/wordpress-develop`](https://github.com/WordPress/wordpress-develop) en moins de deux secondes et [`php-standard-library/php-standard-library`](https://github.com/php-standard-library/php-standard-library) en moins de 200 millisecondes. Les chiffres varient avec le matériel et la taille du projet, mais comme seuil grossier : si Mago prend plus de 30 secondes pour analyser votre projet, quelque chose ne va pas, soit dans Mago, soit dans une entrée pathologique sur laquelle il bute.

Le même principe s'applique à une régression que vous remarquez entre versions. Si une analyse précédemment rapide devient soudainement lente, cela mérite d'être signalé.

Relancez avec `MAGO_LOG=trace` pour obtenir une trace complète du pipeline :

```bash
MAGO_LOG=trace mago analyze
```

Avec le traçage activé, Mago :

- Démarre un surveillant de blocage qui signale tout fichier unique dont l'analyse dure plus de quelques secondes. Utile pour attraper le fichier qui envoie l'analyseur dans une boucle longue ou infinie.
- Imprime les fichiers les plus lents vus pendant la phase d'analyse parallèle, vous pouvez donc voir quelles entrées ont dominé le temps total.
- Émet les durées par phase pour la découverte des sources, la compilation, la fusion de la base de code, le peuplement des métadonnées, l'analyse parallèle, la réduction, etc., vous pouvez donc dire quelle étape est responsable.

Lorsque vous signalez une exécution lente ou une régression, incluez la sortie complète de la trace et le fichier sur lequel le surveillant de blocage pointe. Anonymiser les noms et nettoyer les littéraux sensibles suffit ; nous avons juste besoin de reproduire la forme de l'entrée.
