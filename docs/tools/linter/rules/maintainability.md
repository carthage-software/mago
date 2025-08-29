---
title: Maintainability rules
outline: [2, 3]
---

# Maintainability rules

This document details the rules available in the `Maintainability` category.

| Rule | Code |
| :--- | :---------- |
| Cyclomatic Complexity | [`cyclomatic-complexity`](#cyclomatic-complexity) |
| Excessive Nesting | [`excessive-nesting`](#excessive-nesting) |
| Excessive Parameter List | [`excessive-parameter-list`](#excessive-parameter-list) |
| Halstead | [`halstead`](#halstead) |
| Kan Defect | [`kan-defect`](#kan-defect) |
| No Boolean Flag Parameter | [`no-boolean-flag-parameter`](#no-boolean-flag-parameter) |
| No Else Clause | [`no-else-clause`](#no-else-clause) |
| No Goto | [`no-goto`](#no-goto) |
| Too Many Enum Cases | [`too-many-enum-cases`](#too-many-enum-cases) |
| Too Many Methods | [`too-many-methods`](#too-many-methods) |
| Too Many Properties | [`too-many-properties`](#too-many-properties) |


## <a id="cyclomatic-complexity"></a>`cyclomatic-complexity`

Checks the cyclomatic complexity of classes, traits, enums, interfaces, functions, and closures.

Cyclomatic complexity is a measure of the number of linearly independent paths through a program's source code.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `integer` | `20` |




## <a id="excessive-nesting"></a>`excessive-nesting`

Checks if the nesting level in any block exceeds a configurable threshold.

Deeply nested code is harder to read, understand, and maintain.
Consider refactoring to use early returns, helper methods, or clearer control flow.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `threshold` | `integer` | `7` |

### Examples

#### Correct code

```php
<?php

if ($condition) {
    while ($otherCondition) {
        echo "Hello"; // nesting depth = 2
    }
}
```

#### Incorrect code

```php
<?php

if ($a) {
    if ($b) {
        if ($c) {
            if ($d) {
                if ($e) {
                    if ($f) {
                        if ($g) {
                            if ($h) {
                                echo "Too deeply nested!";
                            }
                        }
                    }
                }
            }
        }
    }
}
```


## <a id="excessive-parameter-list"></a>`excessive-parameter-list`

Detects functions, closures, and methods with too many parameters.

If the number of parameters exceeds a configurable threshold, an issue is reported.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `integer` | `5` |




## <a id="halstead"></a>`halstead`

Computes Halstead metrics (volume, difficulty, effort) and reports if they exceed configured thresholds.

Halstead metrics are calculated by counting operators and operands in the analyzed code.
For more info: https://en.wikipedia.org/wiki/Halstead_complexity_measures



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `volume-threshold` | `double` | `1000` |
| `difficulty-threshold` | `double` | `12` |
| `effort-threshold` | `double` | `7000` |




## <a id="kan-defect"></a>`kan-defect`

Detects classes, traits, interfaces, functions, and closures with high kan defect.

The "Kan Defect" metric is a heuristic for estimating defect proneness in a class or similar structure.
It counts control-flow statements (`while`, `do`, `foreach`, `if`, and `switch`) and sums them using a
formula loosely based on the work of Stephen H. Kan.

References:
  - https://github.com/phpmetrics/PhpMetrics/blob/c43217cd7783bbd54d0b8c1dd43f697bc36ef79d/src/Hal/Metric/Class_/Complexity/KanDefectVisitor.php
  - https://phpmetrics.org/



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `double` | `1.9` |




## <a id="no-boolean-flag-parameter"></a>`no-boolean-flag-parameter`

Flags function-like parameters that use a boolean type.

Boolean flag parameters can indicate a violation of the Single Responsibility Principle (SRP).
Refactor by extracting the flag logic into its own class or method.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

function get_difference(string $a, string $b): string {
    // ...
}

function get_difference_case_insensitive(string $a, string $b): string {
    // ...
}
```

#### Incorrect code

```php
<?php

function get_difference(string $a, string $b, bool $ignore_case): string {
    // ...
}
```


## <a id="no-else-clause"></a>`no-else-clause`

Flags `if` statements that include an `else` or `elseif` branch.

Using `else` or `elseif` can lead to deeply nested code and complex control flow.
This can often be simplified by using early returns (guard clauses), which makes
the code easier to read and maintain by reducing its cyclomatic complexity.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

function process($user) {
    if (!$user->isVerified()) {
        return; // Early return
    }

    // "Happy path" continues here
    $user->login();
}
```

#### Incorrect code

```php
<?php

function process($user) {
    if ($user->isVerified()) {
        // "Happy path" is nested
        $user->login();
    } else {
        // Logic is split across branches
        return;
    }
}
```


## <a id="no-goto"></a>`no-goto`

Detects the use of `goto` statements and labels. The `goto` statement can make
code harder to read, understand, and maintain. It can lead to "spaghetti code"
and make it difficult to follow the flow of execution.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

$i = 0;
while ($i < 10) {
    if ($i === 5) {
        break; // Structured control flow.
    }
    $i++;
}
```

#### Incorrect code

```php
<?php

$i = 0;
loop:
if ($i >= 10) {
    goto end;
}

$i++;
goto loop;
end:
```


## <a id="too-many-enum-cases"></a>`too-many-enum-cases`

Detects enums with too many cases.

This rule checks the number of cases in enums. If the number of cases exceeds a configurable threshold, an issue is reported.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `integer` | `30` |

### Examples

#### Correct code

```php
enum SimpleEnum {
    case A;
    case B;
    case C;
}
```

#### Incorrect code

```php
enum LargeEnum {
    case A;
    case B;
    case C;
    case D;
    case E;
    case F;
    case G;
    case H;
    case I;
    case J;
    case K;
    case L;
    case M;
    case N;
    case O;
    case P;
    case Q;
    case R;
    case S;
    case T;
    case U;
}
```


## <a id="too-many-methods"></a>`too-many-methods`

Detects class-like structures with too many methods.

This rule checks the number of methods in classes, traits, enums, and interfaces.
If the number of methods exceeds a configurable threshold, an issue is reported.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `integer` | `10` |
| `count-hooks` | `boolean` | `false` |
| `count-setters-and-getters` | `boolean` | `false` |

### Examples

#### Correct code

```php
class SimpleClass {
    public function a() {}

    public function b() {}
}
```

#### Incorrect code

```php
class ComplexClass {
    public function a() {}
    public function b() {}
    public function c() {}
    public function d() {}
    public function e() {}
    public function f() {}
    public function g() {}
    public function h() {}
    public function i() {}
    public function j() {}
    public function k() {}
    public function l() {}
    public function m() {}
    public function n() {}
    public function o() {}
    public function p() {}
    public function q() {}
    public function r() {}
    public function s() {}
    public function t() {}
    public function u() {}
}
```


## <a id="too-many-properties"></a>`too-many-properties`

Detects class-like structures with too many properties.

This rule checks the number of properties in classes, traits, and interfaces.
If the number of properties exceeds a configurable threshold, an issue is reported.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `threshold` | `integer` | `10` |

### Examples

#### Correct code

```php
class SimpleClass {
    public $a;
    public $b;
}
```

#### Incorrect code

```php
class ComplexClass {
    public $a; public $b; public $c; public $d; public $e;
    public $f; public $g; public $h; public $i; public $j; public $k;
}
```

