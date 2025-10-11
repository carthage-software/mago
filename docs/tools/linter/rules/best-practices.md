---
title: BestPractices rules
outline: [2, 3]
---

# BestPractices rules

This document details the rules available in the `BestPractices` category.

| Rule | Code |
| :--- | :---------- |
| Combine Consecutive Issets | [`combine-consecutive-issets`](#combine-consecutive-issets) |
| Final Controller | [`final-controller`](#final-controller) |
| Loop Does Not Iterate | [`loop-does-not-iterate`](#loop-does-not-iterate) |
| Middleware In Routes | [`middleware-in-routes`](#middleware-in-routes) |
| No Direct Database Queries | [`no-direct-db-query`](#no-direct-db-query) |
| No ini_set | [`no-ini-set`](#no-ini-set) |
| No Sprintf Concat | [`no-sprintf-concat`](#no-sprintf-concat) |
| Prefer Anonymous Migration | [`prefer-anonymous-migration`](#prefer-anonymous-migration) |
| Prefer Arrow Function | [`prefer-arrow-function`](#prefer-arrow-function) |
| Prefer First Class Callable | [`prefer-first-class-callable`](#prefer-first-class-callable) |
| Prefer Interface | [`prefer-interface`](#prefer-interface) |
| Prefer View Array | [`prefer-view-array`](#prefer-view-array) |
| Prefer While Loop | [`prefer-while-loop`](#prefer-while-loop) |
| Psl Array Functions | [`psl-array-functions`](#psl-array-functions) |
| Psl Data Structures | [`psl-data-structures`](#psl-data-structures) |
| Psl DateTime | [`psl-datetime`](#psl-datetime) |
| Psl Math Functions | [`psl-math-functions`](#psl-math-functions) |
| Psl Output | [`psl-output`](#psl-output) |
| Psl Randomness Functions | [`psl-randomness-functions`](#psl-randomness-functions) |
| Psl Regex Functions | [`psl-regex-functions`](#psl-regex-functions) |
| Psl Sleep Functions | [`psl-sleep-functions`](#psl-sleep-functions) |
| Psl String Functions | [`psl-string-functions`](#psl-string-functions) |
| Use Compound Assignment | [`use-compound-assignment`](#use-compound-assignment) |
| Use WordPress API Functions | [`use-wp-functions`](#use-wp-functions) |
| Yoda Conditions | [`yoda-conditions`](#yoda-conditions) |


## <a id="combine-consecutive-issets"></a>`combine-consecutive-issets`

Suggests combining consecutive calls to `isset()` when they are joined by a logical AND.

For example, `isset($a) && isset($b)` can be turned into `isset($a, $b)`, which is more concise
and avoids repeated function calls. If one or both `isset()` calls are wrapped in parentheses,
the rule will still warn, but it will not attempt an automated fix.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

if (isset($a, $b)) {
    // ...
}
```

#### Incorrect code

```php
<?php

if (isset($a) && isset($b)) {
    // ...
}
```


## <a id="final-controller"></a>`final-controller`

Enforces that controller classes are declared as `final`.

In modern MVC frameworks, controllers should be treated as entry points that orchestrate the application's response to a request. They are not designed to be extension points.

Extending controllers can lead to deep inheritance chains, making the codebase rigid and difficult to maintain. It's a best practice to favor composition (injecting services for shared logic) over inheritance.

If a controller is intended as a base for others, it should be explicitly marked as `abstract`. All other concrete controllers should be `final` to prevent extension.


### Requirements

- **Integrations, any of:**
  - `Symfony`
  - `Laravel`
  - `Tempest`
  - `Spiral`
  - `CakePHP`
  - `Yii`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

namespace App\Http\Controllers;

final class UserController
{
    // ...
}
```

#### Incorrect code

```php
<?php

namespace App\Http\Controllers;

class UserController
{
    // ...
}
```


## <a id="loop-does-not-iterate"></a>`loop-does-not-iterate`

Detects loops (for, foreach, while, do-while) that unconditionally break or return
before executing even a single iteration. Such loops are misleading or redundant
since they give the impression of iteration but never actually do so.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

for ($i = 0; $i < 3; $i++) {
    echo $i;
    if ($some_condition) {
        break; // This break is conditional.
    }
}
```

#### Incorrect code

```php
<?php

for ($i = 0; $i < 3; $i++) {
    break; // The loop never truly iterates, as this break is unconditional.
}
```


## <a id="middleware-in-routes"></a>`middleware-in-routes`

This rule warns against applying middlewares in controllers.

Middlewares should be applied in the routes file, not in the controller.


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

// routes/web.php
Route::get('/user', 'UserController@index')->middleware('auth');
```

#### Incorrect code

```php
<?php

namespace App\Http\Controllers;

class UserController extends Controller
{
    public function __construct()
    {
        $this->middleware('auth');
    }
}
```


## <a id="no-direct-db-query"></a>`no-direct-db-query`

This rule flags all direct method calls on the global `$wpdb` object. Direct database queries
bypass the WordPress object cache, which can lead to poor performance. Using high-level functions
like `get_posts()` is safer and more efficient.


### Requirements

- **Integration:** `WordPress`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$posts = get_posts(['author' => $author_id]);
```

#### Incorrect code

```php
<?php

global $wpdb;
$posts = $wpdb->get_results("SELECT * FROM {$wpdb->posts} WHERE post_author = 1");
```


## <a id="no-ini-set"></a>`no-ini-set`

Enforces that ini_set is not used.

Runtime configuration changes via ini_set make application behavior unpredictable and environment-dependent. They can mask misconfigured servers, introduce subtle bugs, and lead to inconsistent behavior between development, testing, and production environments.

Modern applications should rely on well-defined configuration through php.ini or framework specific configuration. This ensures that configuration is explicit, consistent, and controlled across all environments.

If a setting truly needs to vary between contexts, it should be handled at the infrastructure or framework configuration level, never by calling ini_set within the application code.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

// In framework config files (e.g., wp-config.php), use constants.
define( 'WP_DEBUG', true );

// Use framework-provided functions where available.
wp_raise_memory_limit( 'admin' );
```

#### Incorrect code

```php
<?php

// This can override server settings in an unpredictable way.
ini_set( 'display_errors', 1 );
ini_set( 'memory_limit', '256M' );
```


## <a id="no-sprintf-concat"></a>`no-sprintf-concat`

Disallows string concatenation with the result of an `sprintf` call.

Concatenating with `sprintf` is less efficient and can be less readable than
incorporating the string directly into the format template. This pattern
creates an unnecessary intermediate string and can make the final output
harder to see at a glance.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$name = 'World';
$greeting = sprintf('Hello, %s!', $name);
```

#### Incorrect code

```php
<?php

$name = 'World';
$greeting = 'Hello, ' . sprintf('%s!', $name);
```


## <a id="prefer-anonymous-migration"></a>`prefer-anonymous-migration`

Prefer using anonymous classes for Laravel migrations instead of named classes.
Anonymous classes are more concise and reduce namespace pollution,
making them the recommended approach for migrations.


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

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

class MyMigration extends Migration {
    public function up(): void {
        Schema::create('flights', function (Blueprint $table) {
            $table->id();
            $table->string('name');
            $table->string('airline');
            $table->timestamps();
        });
    }

    public function down(): void {
        Schema::drop('flights');
    }
}

return new MyMigration();
```

#### Incorrect code

```php
<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration {
    public function up(): void {
        Schema::create('flights', function (Blueprint $table) {
            $table->id();
            $table->string('name');
                $table->string('airline');
                $table->timestamps();
        });
    }

    public function down(): void {
        Schema::drop('flights');
    }
};
```


## <a id="prefer-arrow-function"></a>`prefer-arrow-function`

Promotes the use of arrow functions (`fn() => ...`) over traditional closures (`function() { ... }`).

This rule identifies closures that consist solely of a single return statement
and suggests converting them to arrow functions.


### Requirements

- **PHP version:** >= `7.4.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

$a = fn($x) => $x + 1;
```

#### Incorrect code

```php
<?php

$a = function($x) {
    return $x + 1;
};
```


## <a id="prefer-first-class-callable"></a>`prefer-first-class-callable`

Promotes the use of first-class callable syntax (`...`) for creating closures.

This rule identifies closures and arrow functions that do nothing but forward their arguments to another function or method.
In such cases, the more concise and modern first-class callable syntax, introduced in PHP 8.1, can be used instead.
This improves readability by reducing boilerplate code.


### Requirements

- **PHP version:** >= `8.1.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$names = ['Alice', 'Bob', 'Charlie'];
$uppercased_names = array_map(strtoupper(...), $names);
```

#### Incorrect code

```php
<?php

$names = ['Alice', 'Bob', 'Charlie'];
$uppercased_names = array_map(fn($name) => strtoupper($name), $names);
```


## <a id="prefer-interface"></a>`prefer-interface`

Detects when an implementation class is used instead of the interface.


### Requirements

- **Integration:** `Symfony`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

use Symfony\Component\Serializer\SerializerInterface;

class UserController
{
    public function __construct(SerializerInterface $serializer)
    {
        $this->serializer = $serializer;
    }
}
```

#### Incorrect code

```php
<?php

use Symfony\Component\Serializer\Serializer;

class UserController
{
    public function __construct(Serializer $serializer)
    {
        $this->serializer = $serializer;
    }
}
```


## <a id="prefer-view-array"></a>`prefer-view-array`

Prefer passing data to views using the array parameter in the `view()` function,
rather than chaining the `with()` method.`

Using the array parameter directly is more concise and readable.


### Requirements

- **Integration:** `Laravel`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

return view('user.profile', [
    'user' => $user,
    'profile' => $profile,
]);
```

#### Incorrect code

```php
<?php

return view('user.profile')->with([
    'user' => $user,
    'profile' => $profile,
]);
```


## <a id="prefer-while-loop"></a>`prefer-while-loop`

Suggests using a `while` loop instead of a `for` loop when the `for` loop does not have any
initializations or increments. This can make the code more readable and concise.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

while ($i < 10) {
    echo $i;

    $i++;
}
```

#### Incorrect code

```php
<?php

for (; $i < 10;) {
    echo $i;

    $i++;
}
```


## <a id="psl-array-functions"></a>`psl-array-functions`

This rule enforces the usage of Psl array functions over their PHP counterparts.
Psl array functions are preferred because they are type-safe and provide more consistent behavior.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$filtered = Psl\Vec\filter($xs, fn($x) => $x > 2);
```

#### Incorrect code

```php
<?php

$filtered = array_filter($xs, fn($x) => $x > 2);
```


## <a id="psl-data-structures"></a>`psl-data-structures`

This rule enforces the usage of Psl data structures over their SPL counterparts.

Psl data structures are preferred because they are type-safe and provide more consistent behavior.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

declare(strict_types=1);

use Psl\DataStructure\Stack;

$stack = new Stack();
```

#### Incorrect code

```php
<?php

declare(strict_types=1);

$stack = new SplStack();
```


## <a id="psl-datetime"></a>`psl-datetime`

This rule enforces the usage of Psl DateTime classes and functions over their PHP counterparts.

Psl DateTime classes and functions are preferred because they are type-safe and provide more consistent behavior.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$dateTime = new Psl\DateTime\DateTime();
```

#### Incorrect code

```php
<?php

$dateTime = new DateTime();
```


## <a id="psl-math-functions"></a>`psl-math-functions`

This rule enforces the usage of Psl math functions over their PHP counterparts.
Psl math functions are preferred because they are type-safe and provide more consistent behavior.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$abs = Psl\Math\abs($number);
```

#### Incorrect code

```php
<?php

$abs = abs($number);
```


## <a id="psl-output"></a>`psl-output`

This rule enforces the usage of Psl output functions over their PHP counterparts.
Psl output functions are preferred because they are type-safe and provide more consistent behavior.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

Psl\IO\write_line("Hello, world!");
```

#### Incorrect code

```php
<?php

echo "Hello, world!";
```


## <a id="psl-randomness-functions"></a>`psl-randomness-functions`

This rule enforces the usage of Psl randomness functions over their PHP counterparts.

Psl randomness functions are preferred because they are type-safe and provide more consistent behavior.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$randomInt = Psl\SecureRandom\int(0, 10);
```

#### Incorrect code

```php
<?php

$randomInt = random_int(0, 10);
```


## <a id="psl-regex-functions"></a>`psl-regex-functions`

This rule enforces the usage of Psl regex functions over their PHP counterparts.

Psl regex functions are preferred because they are type-safe and provide more consistent behavior.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$result = Psl\Regex\matches('Hello, World!', '/\w+/');
```

#### Incorrect code

```php
<?php

$result = preg_match('/\w+/', 'Hello, World!');
```


## <a id="psl-sleep-functions"></a>`psl-sleep-functions`

This rule enforces the usage of Psl sleep functions over their PHP counterparts.

Psl sleep functions are preferred because they are type-safe, provide more consistent behavior,
and allow other tasks within the event loop to continue executing while the current Fiber pauses.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

use Psl\Async;
use Psl\DateTime;

Async\sleep(DateTime\Duration::seconds(1));
```

#### Incorrect code

```php
<?php

sleep(1);
```


## <a id="psl-string-functions"></a>`psl-string-functions`

                This rule enforces the usage of Psl string functions over their PHP counterparts.

Psl string functions are preferred because they are type-safe and provide more consistent behavior.


### Requirements

- **Integration:** `Psl`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$capitalized = Psl\Str\capitalize($string);
```

#### Incorrect code

```php
<?php

$capitalized = ucfirst($string);
```


## <a id="use-compound-assignment"></a>`use-compound-assignment`

Enforces the use of compound assignment operators (e.g., `+=`, `.=`)
over their more verbose equivalents (`$var = $var + ...`).

Using compound assignments is more concise and idiomatic. For string
concatenation (`.=`), it can also be more performant as it avoids
creating an intermediate copy of the string.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

$count += 1;
$message .= ' Hello';
```

#### Incorrect code

```php
<?php

$count = $count + 1;
$message = $message . ' Hello';
```


## <a id="use-wp-functions"></a>`use-wp-functions`

This rule encourages using WordPress's wrapper functions instead of native PHP functions for
common tasks like HTTP requests, filesystem operations, and data handling. The WordPress APIs
provide a consistent, secure, and reliable abstraction that works across different hosting
environments.


### Requirements

- **Integration:** `WordPress`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

// For remote requests:
$response = wp_remote_get('https://example.com/api/data');

// For filesystem operations:
global $wp_filesystem;
require_once ABSPATH . 'wp-admin/includes/file.php';
WP_Filesystem();
$wp_filesystem->put_contents( '/path/to/my-file.txt', 'data' );
```

#### Incorrect code

```php
<?php

// For remote requests:
$ch = curl_init();
curl_setopt($ch, CURLOPT_URL, 'https://example.com/api/data');
// ...

// For filesystem operations:
file_put_contents('/path/to/my-file.txt', 'data');
```


## <a id="yoda-conditions"></a>`yoda-conditions`

This rule enforces the use of "Yoda" conditions for comparisons. The variable should always be
on the right side of the comparison, while the constant, literal, or function call is on the left.
This prevents the common bug of accidentally using an assignment (`=`) instead of a comparison (`==`),
which would cause a fatal error in a Yoda condition instead of a silent logical bug.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

if ( true === $is_active ) { /* ... */ }
if ( 5 === $count ) { /* ... */ }
```

#### Incorrect code

```php
<?php

// Vulnerable to the accidental assignment bug, e.g., if ($is_active = true).
if ( $is_active === true ) { /* ... */ }
```

