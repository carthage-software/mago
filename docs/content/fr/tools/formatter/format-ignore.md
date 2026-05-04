+++
title = "Format ignore"
description = "Marqueurs de commentaire qui exemptent du code du formatage au niveau du fichier, de la région ou d'une seule instruction."
nav_order = 30
nav_section = "Tools"
nav_subsection = "Formatter"
+++
# Format ignore

Parfois, vous devez conserver un formatage que le formateur changerait autrement : matrices alignées, art ASCII, extraits hérités que vous ne voulez pas voir touchés. Mago dispose de trois pragmas pour cela.

## Les pragmas

| Marqueur | Portée | Effet |
| :--- | :--- | :--- |
| `@mago-format-ignore` | Fichier | Ignore le formatage pour tout le fichier. |
| `@mago-format-ignore-next` | Une instruction ou un membre de classe | Préserve l'instruction ou le membre suivant exactement. |
| `@mago-format-ignore-start` / `@mago-format-ignore-end` | Région | Préserve tout entre les deux marqueurs. |

Les trois acceptent également le préfixe plus long `@mago-formatter-`, donc `@mago-formatter-ignore-next` est identique à `@mago-format-ignore-next`.

## Ignore au niveau du fichier

Placez un commentaire contenant `@mago-format-ignore` n'importe où dans le fichier et le formateur laisse le fichier entier intact.

```php
<?php
// @mago-format-ignore

$a=1; $b=2; $c=3;
```

## Ignorer l'instruction suivante

Utile lorsque vous voulez qu'une construction soigneusement alignée survive intacte. Le marqueur préserve l'instruction ou le membre de classe immédiatement suivant.

```php
<?php

$formatted = 'normal';

// @mago-format-ignore-next
const GRID = [
    [1, 2, 3], [1, 2, ], [0, 0],
    [0, 0],    [1, 3],   [1, 1, 0]
];

$alsoFormatted = 'normal';
```

Le même marqueur fonctionne sur tous types de membres de classe :

```php
<?php

class Example
{
    // @mago-format-ignore-next
    public const MATRIX = [[1,2], [3,4]];

    // @mago-format-ignore-next
    public $alignedProperty = 123;

    // @mago-format-ignore-next
    public function preservedMethod() { return 1; }
}
```

Il fonctionne aussi dans les traits, interfaces et enums :

```php
<?php

enum Status: int
{
    // @mago-format-ignore-next
    case PENDING = 1;

    case Active = 2;
}

interface MyInterface
{
    // @mago-format-ignore-next
    public function foo( $a , $b ) ;
}

trait MyTrait
{
    // @mago-format-ignore-next
    public $prop = 123;
}
```

## Ignorer une région

Pour les blocs plus larges, encadrez la région avec les marqueurs de début et de fin :

```php
<?php

$formatted = 'normal';

// @mago-format-ignore-start
$a=1;
$b=2;
$c=3;
// @mago-format-ignore-end

$alsoFormatted = 'normal';
```

Les marqueurs de région fonctionnent aussi à l'intérieur des corps de classe :

```php
<?php

class Example
{
    public const FORMATTED = 1;

    // @mago-format-ignore-start
    public const A = 1;
    public const B = 2;
    public $prop = 123;
    public function foo() { return 1; }
    // @mago-format-ignore-end

    public const ALSO_FORMATTED = 3;
}
```

Si une région est ouverte mais jamais fermée, le formateur préserve tout depuis le marqueur de début jusqu'à la fin du fichier.

## Style de commentaire

Tous les marqueurs fonctionnent avec tous les styles de commentaires PHP :

```php
<?php

// @mago-format-ignore-next
$lineComment = 1;

/* @mago-format-ignore-next */
$blockComment = 2;

/**
 * @mago-format-ignore-next
 */
$docblock = 3;

# @mago-format-ignore-next
$hashComment = 4;
```

## Usages courants

### Tables de données alignées

```php
<?php

// @mago-format-ignore-next
const LOOKUP_TABLE = [
    'short'     => 1,
    'medium'    => 10,
    'long'      => 100,
    'very_long' => 1000,
];
```

### Matrices

```php
<?php

// @mago-format-ignore-next
const TRANSFORMATION_MATRIX = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
```

### Art ASCII

```php
<?php

// @mago-format-ignore-start
/*
 *   _____
 *  /     \
 * | () () |
 *  \  ^  /
 *   |||||
 */
// @mago-format-ignore-end
```

### Code hérité

```php
<?php

// @mago-format-ignore-start
function old_function($a,$b,$c) {
    return $a+$b+$c;
}
// @mago-format-ignore-end
```

## Règles de linter

Le linter a deux règles qui détectent les placements inefficaces :

- `ineffective-format-ignore-region` : détecte les marqueurs `@mago-format-ignore-start` à l'intérieur d'une expression (un littéral de tableau, une liste d'arguments d'appel de fonction) où le marqueur ne peut pas influencer le formatage.
- `ineffective-format-ignore-next` : détecte les marqueurs `@mago-format-ignore-next` à l'intérieur d'une expression plutôt qu'avant une instruction ou un membre.

Le mauvais cas ressemble à ceci :

```php
<?php

// Le marqueur est à l'intérieur d'un littéral de tableau, sans effet.
$arr = [ // @mago-format-ignore-next
    1,
    2,
    3
];
```

La correction est de mettre le marqueur avant l'instruction entière :

```php
<?php

// @mago-format-ignore-next
$arr = [
    1, 2, 3,
];
```
