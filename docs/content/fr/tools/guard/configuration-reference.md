+++
title = "Référence de configuration du guard"
description = "Toutes les options que Mago accepte sous [guard]."
nav_order = 40
nav_section = "Tools"
nav_subsection = "Guard"
+++
# Référence de configuration

Les paramètres se trouvent sous `[guard]` dans `mago.toml`. La configuration a deux parties : `[guard.perimeter]` pour les règles de dépendance et `[[guard.structural.rules]]` pour les conventions structurelles.

## Options de niveau supérieur

| Option | Type | Défaut | Description |
| :--- | :--- | :--- | :--- |
| `mode` | `string` | `"default"` | Quelles vérifications exécuter. L'un de `"default"`, `"structural"`, `"perimeter"`. |
| `excludes` | `string[]` | `[]` | Chemins ou motifs glob à exclure de l'analyse. S'ajoute à `[source].excludes`. |
| `baseline` | `string` | non défini | Chemin vers un fichier de baseline. Équivalent à passer `--baseline` à chaque exécution. |
| `baseline-variant` | `string` | `"loose"` | Format pour les baselines nouvellement générées. `"loose"` (basée sur le compte) ou `"strict"` (correspondance exacte de ligne). Voir [baseline](/fundamentals/baseline/). |
| `minimum-fail-level` | `string` | `"error"` | Sévérité minimale qui provoque un code de sortie non nul. L'un de `"note"`, `"help"`, `"warning"`, `"error"`. Remplacé par `--minimum-fail-level`. |

`mode` contrôle quelle moitié du guard s'exécute :

- `"default"` exécute les deux moitiés.
- `"structural"` exécute uniquement les vérifications structurelles.
- `"perimeter"` exécute uniquement les vérifications de périmètre.

```toml
[guard]
mode = "structural"
```

Les indicateurs `--structural` et `--perimeter` remplacent le mode configuré. Voir la [référence de commande](/tools/guard/command-reference/).

`excludes` ici s'ajoute à ce que vous avez défini dans `[source].excludes` ; il ne réduit jamais la liste globale.

```toml
[source]
excludes = ["cache/**"]

[guard]
excludes = ["src/ThirdParty/**"]
```

## Guard de périmètre

La section périmètre définit les règles de dépendance entre parties du projet.

```toml
[guard.perimeter]
layering = [
    "CarthageSoftware\\Domain",
    "CarthageSoftware\\Application",
    "CarthageSoftware\\UI",
    "CarthageSoftware\\Infrastructure",
]

[guard.perimeter.layers]
core = ["@native", "Psl\\**"]
psr = ["Psr\\**"]
framework = ["Symfony\\**", "Doctrine\\**"]

[[guard.perimeter.rules]]
namespace = "CarthageSoftware\\Domain"
permit = ["@layer:core"]

[[guard.perimeter.rules]]
namespace = "CarthageSoftware\\Application"
permit = ["@layer:core", "@layer:psr"]

[[guard.perimeter.rules]]
namespace = "CarthageSoftware\\Infrastructure"
permit = ["@layer:core", "@layer:psr", "@layer:framework"]

[[guard.perimeter.rules]]
namespace = "CarthageSoftware\\Tests"
permit = ["@all"]
```

### `layering`

Une liste ordonnée d'espaces de noms, du cœur le plus indépendant jusqu'à la couche la plus externe. Chaque couche ne peut dépendre que des couches définies avant elle. Une dépendance qui pointe vers une couche plus externe déclenche une violation.

### Alias de couche

`[guard.perimeter.layers]` définit des groupes réutilisables d'espaces de noms et de chemins, référencés depuis les règles avec `@layer:<name>`.

### Règles

Chaque table `[[guard.perimeter.rules]]` définit une règle :

- `namespace` : l'espace de noms auquel cette règle s'applique. Soit un espace de noms se terminant par `\`, soit le mot-clé spécial `@global` pour l'espace de noms global.
- `permit` : les dépendances autorisées. Soit une liste de chaînes, soit une liste d'objets détaillés.

#### Valeurs `permit`

`permit` accepte des chemins. Un chemin peut être un mot-clé, un espace de noms, un symbole ou un motif glob.

| Chemin | Description |
| :--- | :--- |
| `@global` | Symboles définis dans l'espace de noms global. |
| `@all` | Tout symbole n'importe où dans le projet, y compris les paquets vendor. Utile pour les tests. |
| `@self` / `@this` | Tout symbole dans le même espace de noms racine que le `namespace` de la règle. |
| `@native` / `@php` | Fonctions, classes et constantes intégrées de PHP. |
| `@layer:<name>` | Tous les espaces de noms et chemins dans l'alias nommé de `[guard.perimeter.layers]`. |
| `App\Shared\\**` | Motif glob. `*` correspond à un seul segment d'espace de noms, `**` correspond à zéro ou plus. |
| `App\Service` | Nom de symbole pleinement qualifié exact. |
| `App\Service\\` | Espace de noms exact. Autorise les symboles directement à l'intérieur. |

Vous pouvez restreindre une permission par type de symbole en utilisant une forme objet :

```toml
[[guard.perimeter.rules]]
namespace = "DoctrineMigrations\\"
permit = [{ path = "@all", kinds = ["class-like"] }]
```

- `path` : n'importe quelle des formes de chemin ci-dessus.
- `kinds` : quels types de symboles sont autorisés. Les valeurs sont `class-like` (couvre les classes, interfaces, traits, enums), `function`, `constant` et `attribute`.

## Guard structurel

`[[guard.structural.rules]]` définit les conventions structurelles. Chaque entrée combine des sélecteurs qui choisissent quels symboles inspecter avec des contraintes que les symboles sélectionnés doivent satisfaire.

```toml
[[guard.structural.rules]]
on = "CarthageSoftware\\UI\\**\\Controller\\**"
target = "class"
must-be-named = "*Controller"
must-be-final = true
must-be-readonly = true
reason = "Controllers must be final and follow naming conventions."

[[guard.structural.rules]]
on = "CarthageSoftware\\Domain\\**\\Repository\\**"
target = "interface"
must-be-named = "*RepositoryInterface"
reason = "Domain repository interfaces must follow a standard naming convention."

[[guard.structural.rules]]
on = "CarthageSoftware\\Infrastructure\\**\\Repository\\**"
target = "class"
must-be-final = true
must-extend = "CarthageSoftware\\Infrastructure\\Shared\\Repository\\AbstractRepository"
reason = "Infrastructure repositories must extend our abstract class."

[[guard.structural.rules]]
on = "CarthageSoftware\\Domain\\**\\Enum\\**"
must-be = ["enum"]
reason = "This namespace is designated for enums only."
```

### Sélecteurs

| Clé | Description |
| :--- | :--- |
| `on` | Requis. Motif glob correspondant au nom pleinement qualifié des symboles auxquels cette règle s'applique. |
| `not-on` | Motif glob optionnel excluant les symboles qui correspondraient autrement à `on`. |
| `target` | Filtre optionnel restreignant la règle à un type de symbole. L'un de `class`, `interface`, `trait`, `enum`, `function`, `constant`. |

### Contraintes

| Clé | Description |
| :--- | :--- |
| `must-be` | Restreint l'espace de noms sélectionné à ne contenir que les types de symboles listés. Valeurs : `class`, `interface`, `trait`, `enum`, `function`, `constant`. |
| `must-be-named` | Motif glob auquel le nom du symbole doit correspondre (par exemple `*Controller`). |
| `must-be-final` | Booléen. `true` exige `final` ; `false` l'interdit. |
| `must-be-abstract` | Booléen. `true` exige `abstract` ; `false` l'interdit. |
| `must-be-readonly` | Booléen. `true` exige `readonly` ; `false` l'interdit. |
| `must-implement` | Une ou plusieurs interfaces que la classe doit implémenter. |
| `must-extend` | Une classe que le symbole doit étendre. |
| `must-use-trait` | Un ou plusieurs traits que le symbole doit utiliser. |
| `must-use-attribute` | Un ou plusieurs attributs que le symbole doit porter. |
| `reason` | Explication lisible affichée dans les messages d'erreur. |

#### Formes des contraintes d'héritage

`must-implement`, `must-extend`, `must-use-trait` et `must-use-attribute` acceptent une seule chaîne, un tableau de chaînes (AND) ou un tableau de tableaux de chaînes (OR de AND). Le littéral `"@nothing"` interdit toute valeur.

```toml
must-extend = "App\\BaseClass"

must-implement = ["App\\InterfaceA", "App\\InterfaceB"]

must-extend = [
    ["App\\AbstractA", "App\\AbstractB"],
    ["App\\AbstractC"],
]

must-implement = "@nothing"
```
