+++
title = "Supprimer les problèmes"
description = "Comment utiliser les pragmas @mago-expect et @mago-ignore pour réduire au silence des problèmes spécifiques dans votre code."
nav_order = 30
nav_section = "Fondamentaux"
+++
# Supprimer les problèmes

Corriger le problème sous-jacent est presque toujours la bonne réponse. Parfois ce n'est pas le cas : code legacy, faux positifs, exceptions délibérées. Pour ces cas-là, Mago dispose de deux pragmas en commentaire à mettre dans le code : `@mago-expect` et `@mago-ignore`.

Les deux prennent la forme `category:code`, avec trois catégories disponibles :

- `lint` (alias `linter`) pour les problèmes du linter.
- `analysis` (alias `analyzer`, `analyser`) pour les problèmes de l'analyseur.
- `guard` pour les problèmes du guard architectural.

Plusieurs codes peuvent être supprimés à la fois avec une liste séparée par des virgules, et un raccourci de comptage `(N)` traite le cas où le même code se déclenche N fois sur une ligne.

## `@mago-expect`

Affirme qu'un problème spécifique est attendu sur la ligne qui suit. Le plus strict des deux pragmas, et celui que nous recommandons par défaut.

```php
// @mago-expect lint:no-shorthand-ternary
$result = $value ?: 'default';
```

Plusieurs codes :

```php
// @mago-expect lint:no-shorthand-ternary,unused-variable
$result = $value ?: 'default';
```

Si chaque code attendu correspond à un problème réel, les problèmes sont supprimés. Si un code attendu ne correspond à rien, Mago signale un avertissement `unfulfilled-expect` afin que le pragma ne reste pas silencieusement après que le code sous-jacent ait été corrigé.

## `@mago-ignore`

Supprime les codes listés sur la ligne ou le bloc qui suit, mais ne signale pas bruyamment quand le code n'est plus déclenché. Mago signale tout de même une remarque `unused-pragma` afin que vous puissiez nettoyer, mais seulement au niveau remarque plutôt qu'avertissement.

```php
// @mago-ignore lint:no-shorthand-ternary
$result = $value ?: 'default';
```

Plusieurs codes fonctionnent de la même manière :

```php
// @mago-ignore lint:no-shorthand-ternary,no-assign-in-condition
if ($result = $value ?: 'default') {
    // Do something with $result
}
```

## Suppression au niveau du bloc

Quand un pragma est sur la ligne avant un bloc (fonction, classe, `if`, …), il couvre tout le bloc.

```php
// @mago-ignore analysis:missing-return-statement
function foo(): string {
    if (rand(0, 1)) {
        return 'foo';
    }
    // No return statement here.
}
```

Idem pour les listes multi-codes :

```php
// @mago-ignore analysis:missing-return-statement,unreachable-code
function foo(): string {
    if (rand(0, 1)) {
        return 'foo';
        echo 'This code is unreachable';
    }
}
```

## Supprimer N occurrences

Quand une seule ligne (ou un bloc couvert par un pragma de portée) déclenche le même code plusieurs fois, répéter le code est fastidieux :

```php
// @mago-expect analysis:mixed-operand,mixed-operand,mixed-operand
return $a . $b . $c;
```

Utilisez plutôt le raccourci `(N)` :

```php
// @mago-expect analysis:mixed-operand(3)
return $a . $b . $c;
```

`N` doit être un entier positif. `code(1)` équivaut à un simple `code`. Les suffixes mal formés comme `(0)`, `(abc)` ou des parenthèses non équilibrées sont traités comme faisant partie du nom du code et ne correspondront pas.

Le comptage se mélange aux listes virgule normales :

```php
// @mago-expect analysis:mixed-operand(2),unused-variable
```

### Comptes non satisfaits

Si moins de problèmes correspondent qu'attendu, Mago signale `unfulfilled-expect` et la correction automatique réduit le compte plutôt que de supprimer la directive (ce qui réactiverait les problèmes qui correspondaient) :

```php
// Before: 3 matches expected, only 2 happened.
// @mago-expect analysis:mixed-operand(3)
return $a . $b;

// After auto-fix: count drops so the 2 real matches stay suppressed.
// @mago-expect analysis:mixed-operand(2)
return $a . $b;
```

### Sémantique ligne vs bloc

Pour les pragmas au niveau ligne, un code nu supprime au plus une occurrence ; `(N)` relève le plafond à `N`.

Pour les pragmas au niveau bloc (de portée), un code nu supprime chaque occurrence correspondante du bloc. Ajouter `(N)` plafonne la suppression à `N` correspondances, donc la `N+1`-ème correspondance est tout de même signalée. Utile quand vous voulez vous assurer qu'aucun problème supplémentaire n'apparaît.

## Tout supprimer : `all`

Le code spécial `all` supprime tout d'un coup. À utiliser avec parcimonie : il masque aussi tout nouveau code ajouté plus tard.

Au sein d'une seule catégorie :

```php
// @mago-ignore lint:all
$result = $value ?: ($x == true ? 'yes' : 'no');

// @mago-expect analysis:all
function legacy_code(): string {
    if (rand(0, 1)) {
        return 'foo';
    }
}
```

Toutes catégories confondues :

```php
// @mago-ignore all
$result = eval($value) ?: 'default';
```

Dans un docblock au-dessus d'un bloc, cela couvre tout le bloc :

```php
/**
 * @mago-ignore all
 */
function legacy_code(): string {
    // Every linter, analyzer, and guard issue is suppressed here.
}
```

Préférez des codes spécifiques quand vous le pouvez. `all` est un instrument grossier qui masque les nouveaux problèmes que vous voudriez voir.

## Choisir entre expect et ignore

- `@mago-expect` est le bon choix par défaut. Il garantit que vous serez averti une fois le problème sous-jacent corrigé.
- `@mago-ignore` est l'option plus légère pour les suppressions moins critiques ou quand vous acceptez que le pragma puisse survivre silencieusement au problème.

## Exemples

```php
// Suppress a guard issue
// @mago-expect guard:disallowed-use
use App\Infrastructure\SomeForbiddenClass;

// Suppress one lint issue
// @mago-expect lint:no-shorthand-ternary
$result = $condition ?: 'default';

// Suppress issues for an entire function
// @mago-expect analysis:missing-return-statement,impossible-condition
function complexFunction(): string {
    if (false) {
        return 'never reached';
    }
}

// Three occurrences of one code on the next line
// @mago-expect analysis:mixed-operand(3)
return $a . $b . $c;

// All lint issues on one line
// @mago-ignore lint:all
$result = $value ?: ($x == true ? 'yes' : 'no');

// Everything, for a legacy function
// @mago-ignore all
function legacyFunction(): string {
    // Everything suppressed here.
}
```
