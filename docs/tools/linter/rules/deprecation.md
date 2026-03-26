---
title: Deprecation rules
outline: [2, 3]
---

# Deprecation rules

This document details the rules available in the `Deprecation` category.

| Rule | Code |
| :--- | :---------- |
| Deprecated Cast | [`deprecated-cast`](#deprecated-cast) |
| Deprecated Shell Execute String | [`deprecated-shell-execute-string`](#deprecated-shell-execute-string) |
| Deprecated Switch Semicolon | [`deprecated-switch-semicolon`](#deprecated-switch-semicolon) |
| Explicit Nullable Param | [`explicit-nullable-param`](#explicit-nullable-param) |
| No Underscore Class | [`no-underscore-class`](#no-underscore-class) |
| No Void Reference Return | [`no-void-reference-return`](#no-void-reference-return) |
| Optional Parameter Before Required | [`optional-param-order`](#optional-param-order) |


## <a id="deprecated-cast"></a>`deprecated-cast`

Detect the usage of deprecated type casts in PHP code.

In PHP 8.5, the following type casts have been deprecated:

- `(integer)`: The integer cast has been deprecated in favor of `(int)`.
- `(boolean)`: The boolean cast has been deprecated in favor of `(bool)`.
- `(double)`: The double cast has been deprecated in favor of `(float)`.
- `(binary)`: The binary cast has been deprecated in favor of `(string)`.


### Requirements

- **PHP version:** >= `8.5.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

(int) $value;
```

#### Incorrect code

```php
<?php

(integer) $value;
```


## <a id="deprecated-shell-execute-string"></a>`deprecated-shell-execute-string`

Detect the usage of deprecated shell execute strings in PHP code.

In PHP 8.5, the shell execute string syntax (enclosed in backticks, e.g., `` `ls -l` ``) has been deprecated.

This rule identifies instances of shell execute strings and provides guidance on how to replace them with safer alternatives,
such as using the `shell_exec()` function or other appropriate methods for executing shell commands.


### Requirements

- **PHP version:** >= `8.5.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

shell_exec('ls -l');
```

#### Incorrect code

```php
<?php

`ls -l`;
```


## <a id="deprecated-switch-semicolon"></a>`deprecated-switch-semicolon`

Detect the usage of semicolon as a switch case separator.

In PHP 8.5, the use of a semicolon (`;`) as a case separator in switch statements has been deprecated.

Instead, the colon (`:`) should be used to separate case statements.


### Requirements

- **PHP version:** >= `8.5.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

switch ($value) {
    case 1:
        // code for case 1
        break;
    case 2:
        // code for case 2
        break;
    default:
        // default case
        break;
}
```

#### Incorrect code

```php
<?php

switch ($value) {
    case 1;
        // code for case 1
        break;
    case 2;
        // code for case 2
        break;
    default;
        // default case
        break;
}
```


## <a id="explicit-nullable-param"></a>`explicit-nullable-param`

Detects parameters that are implicitly nullable and rely on a deprecated feature.

Such parameters are considered deprecated; an explicit nullable type hint is recommended.


### Requirements

- **PHP version:** >= `8.4.0`

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

- **PHP version:** >= `8.4.0`

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

- **PHP version:** >= `8.2.0`

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

- **PHP version:** >= `8.0.0`

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

