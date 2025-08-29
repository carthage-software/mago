---
title: Deprecation rules
outline: [2, 3]
---

# Deprecation rules

This document details the rules available in the `Deprecation` category.

| Rule | Code |
| :--- | :---------- |
| Explicit Nullable Param | [`explicit-nullable-param`](#explicit-nullable-param) |
| No Underscore Class | [`no-underscore-class`](#no-underscore-class) |
| No Void Reference Return | [`no-void-reference-return`](#no-void-reference-return) |
| Optional Parameter Before Required | [`optional-param-order`](#optional-param-order) |


## <a id="explicit-nullable-param"></a>`explicit-nullable-param`

Detects parameters that are implicitly nullable and rely on a deprecated feature.

Such parameters are considered deprecated; an explicit nullable type hint is recommended.


### Requirements

- **PHP version:** PHP `>= 8.4.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

function foo(?string $param) {}

function bar(null|string $param) {}

function baz(null|object $param = null) {}
```

#### Incorrect code

```php
<?php

function foo(string $param = null) {}

function bar(string $param = NULL) {}

function baz(object $param = null) {}
```


## <a id="no-underscore-class"></a>`no-underscore-class`

Detects class, interface, trait, or enum declarations named `_`.

Such names are considered deprecated; a more descriptive identifier is recommended.


### Requirements

- **PHP version:** PHP `>= 8.4.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

class MyService {}
```

#### Incorrect code

```php
<?php

class _ {}
```


## <a id="no-void-reference-return"></a>`no-void-reference-return`

Detects functions, methods, closures, arrow functions, and set property hooks that return by reference from a void function.
Such functions are considered deprecated; returning by reference from a void function is deprecated since PHP 8.0.


### Requirements

- **PHP version:** PHP `>= 8.2.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

function &foo(): string {
    // ...
}
```

#### Incorrect code

```php
<?php

function &foo(): void {
    // ...
}
```


## <a id="optional-param-order"></a>`optional-param-order`

                Detects optional parameters defined before required parameters in function-like declarations.
Such parameter order is considered deprecated; required parameters should precede optional parameters.


### Requirements

- **PHP version:** PHP `>= 8.0.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

function foo(string $required, ?string $optional = null): void {}
```

#### Incorrect code

```php
<?php

function foo(?string $optional = null, string $required): void {}
```

