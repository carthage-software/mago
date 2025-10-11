---
title: Safety rules
outline: [2, 3]
---

# Safety rules

This document details the rules available in the `Safety` category.

| Rule | Code |
| :--- | :---------- |
| No Error Control Operator | [`no-error-control-operator`](#no-error-control-operator) |
| No Eval | [`no-eval`](#no-eval) |
| No FFI | [`no-ffi`](#no-ffi) |
| No Global | [`no-global`](#no-global) |
| No Request All | [`no-request-all`](#no-request-all) |
| No Request Variable | [`no-request-variable`](#no-request-variable) |
| No Shell Execute String | [`no-shell-execute-string`](#no-shell-execute-string) |
| No Unsafe Finally | [`no-unsafe-finally`](#no-unsafe-finally) |


## <a id="no-error-control-operator"></a>`no-error-control-operator`

Detects the use of the error control operator `@`.

The error control operator suppresses errors and makes debugging more difficult.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

try {
    $result = file_get_contents('example.txt');
} catch (Throwable $e) {
    // Handle error
}
```

#### Incorrect code

```php
<?php

$result = @file_get_contents('example.txt');
```


## <a id="no-eval"></a>`no-eval`

Detects unsafe uses of the `eval` construct.
The `eval` construct executes arbitrary code, which can be a major security risk if not used carefully.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

// Safe alternative to eval
$result = json_decode($jsonString);
```

#### Incorrect code

```php
<?php

eval('echo "Hello, world!";');
```


## <a id="no-ffi"></a>`no-ffi`

Detects unsafe use of the PHP FFI (Foreign Function Interface) extension.

The FFI extension allows interaction with code written in other languages, such as C, C++, and Rust.
This can introduce potential security risks and stability issues if not handled carefully.

If you are confident in your use of FFI and understand the risks, you can disable this rule in your Mago configuration.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

// Using a safe alternative to FFI
$data = 'some data';
$hash = hash('sha256', $data);
```

#### Incorrect code

```php
<?php

use FFI;

$ffi = FFI::cdef("void* malloc(size_t size);");
$ffi->malloc(1024); // Allocate memory but never free it
```


## <a id="no-global"></a>`no-global`

Detects the use of the `global` keyword and the `$GLOBALS` variable.

The `global` keyword introduces global state into your function, making it harder to reason about and test.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

function foo(string $bar): void {
    // ...
}
```

#### Incorrect code

```php
<?php

function foo(): void {
    global $bar;
    // ...
}
```


## <a id="no-request-all"></a>`no-request-all`

Detects the use of `$request->all()` or `Request::all()` in Laravel applications.

Such calls retrieve all input values, including ones you might not expect or intend to handle.
It is recommended to use `$request->only([...])` to specify the inputs you need explicitly, ensuring better security and validation.


### Requirements

- **Integration:** `Laravel`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

namespace App\Http\Controllers;

use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;

class UserController extends Controller
{
    /**
     * Store a new user.
     */
    public function store(Request $request): RedirectResponse
    {
        $data = $request->only(['name', 'email', 'password']);

        // ...
    }
}
```

#### Incorrect code

```php
<?php

namespace App\Http\Controllers;

use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;

class UserController extends Controller
{
    /**
     * Store a new user.
     */
    public function store(Request $request): RedirectResponse
    {
        $data = $request->all();

        // ...
    }
}
```


## <a id="no-request-variable"></a>`no-request-variable`

Detects the use of the `$_REQUEST` variable, which is considered unsafe.

Use `$_GET`, `$_POST`, or `$_COOKIE` instead for better clarity.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

$identifier = $_GET['id'];
```

#### Incorrect code

```php
<?php

$identifier = $_REQUEST['id'];
```


## <a id="no-shell-execute-string"></a>`no-shell-execute-string`

Detects the use of shell execute strings (`...`) in PHP code.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

$output = shell_exec('ls -l');
```

#### Incorrect code

```php
<?php

$output = `ls -l`;
```


## <a id="no-unsafe-finally"></a>`no-unsafe-finally`

Detects control flow statements in `finally` blocks.

Control flow statements in `finally` blocks override control flows from `try` and `catch` blocks,
leading to unexpected behavior.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

function example(): int {
    try {
        return get_value();
    } finally {
        // no control flow statements
    }
}
```

#### Incorrect code

```php
<?php

function example(): int {
    try {
        return get_value();
    } finally {
        return 42; // Unsafe control flow statement in finally block
    }
}
```

