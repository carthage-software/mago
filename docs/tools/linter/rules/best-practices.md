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
| No array_merge In Loop | [`no-array-merge-in-loop`](#no-array-merge-in-loop) |
| No Direct Database Queries | [`no-direct-db-query`](#no-direct-db-query) |
| No Direct Exception Throw | [`no-direct-exception-throw`](#no-direct-exception-throw) |
| No ini_set | [`no-ini-set`](#no-ini-set) |
| No Inline | [`no-inline`](#no-inline) |
| No Literal Namespace String | [`no-literal-namespace-string`](#no-literal-namespace-string) |
| No ObjectManager Singleton | [`no-object-manager-singleton`](#no-object-manager-singleton) |
| No ObjectManagerInterface Type Hint | [`no-object-manager-type-hint`](#no-object-manager-type-hint) |
| No Proxy/Interceptor In Constructor | [`no-proxy-interceptor-in-constructor`](#no-proxy-interceptor-in-constructor) |
| No Sprintf Concat | [`no-sprintf-concat`](#no-sprintf-concat) |
| No Test Namespace Import | [`no-test-namespace-import`](#no-test-namespace-import) |
| No $this In Template | [`no-this-in-template`](#no-this-in-template) |
| Prefer Anonymous Migration | [`prefer-anonymous-migration`](#prefer-anonymous-migration) |
| Prefer Arrow Function | [`prefer-arrow-function`](#prefer-arrow-function) |
| Prefer Early Continue | [`prefer-early-continue`](#prefer-early-continue) |
| Prefer First Class Callable | [`prefer-first-class-callable`](#prefer-first-class-callable) |
| Prefer Interface | [`prefer-interface`](#prefer-interface) |
| Prefer Static Closure | [`prefer-static-closure`](#prefer-static-closure) |
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
| Require Namespace | [`require-namespace`](#require-namespace) |
| Single Class Per File | [`single-class-per-file`](#single-class-per-file) |
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


## <a id="no-array-merge-in-loop"></a>`no-array-merge-in-loop`

Flags `array_merge()` calls inside `foreach`, `for`, `while`, and `do-while` loops.
Calling `array_merge()` in a loop causes quadratic time complexity because it copies
the entire array on each iteration. Use spread operator or collect values and merge once.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$collected = [];
foreach ($items as $item) {
    $collected[] = $item;
}
$result = array_merge($base, $collected);
```

#### Incorrect code

```php
<?php

$result = [];
foreach ($items as $item) {
    $result = array_merge($result, $item);
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


## <a id="no-direct-exception-throw"></a>`no-direct-exception-throw`

Flags direct throwing of the generic `\Exception` base class. Use context-specific
exception types instead (e.g. `InvalidArgumentException`, `RuntimeException`,
or custom exception classes) for better error handling and debugging.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

throw new \InvalidArgumentException('Invalid value');
```

#### Incorrect code

```php
<?php

throw new \Exception('Something went wrong');
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


## <a id="no-inline"></a>`no-inline`

Disallows inline content (text outside of PHP tags) in source files.

Most modern PHP applications are source-code only and do not use PHP as a templating
language. Inline content before `<?php`, after `?>`, or between PHP tags is typically
unintentional and can cause issues such as unexpected output or "headers already sent"
errors.

This rule is disabled by default and is intended for codebases that do not use PHP
templates.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

namespace App;

echo "Hello, world!";
```

#### Incorrect code

```php
Hello
<?php

echo "Hello, world!";

?>
Goodbye
```


## <a id="no-literal-namespace-string"></a>`no-literal-namespace-string`

Flags hardcoded fully qualified class name strings. Use `::class` notation
instead for better IDE support, refactoring safety, and static analysis.


### Requirements

- **Integration:** `Magento`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

$className = \Magento\Catalog\Model\Product::class;
```

#### Incorrect code

```php
<?php

$className = 'Magento\Catalog\Model\Product';
```


## <a id="no-object-manager-singleton"></a>`no-object-manager-singleton`

Flags direct usage of `ObjectManager::getInstance()`. In Magento 2, the ObjectManager
singleton should never be used directly. Dependencies should be injected via constructor
dependency injection instead.


### Requirements

- **Integration:** `Magento`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

namespace Vendor\Module\Model;

use Magento\Catalog\Api\ProductRepositoryInterface;

class Example
{
    public function __construct(
        private ProductRepositoryInterface $productRepository,
    ) {}
}
```

#### Incorrect code

```php
<?php

namespace Vendor\Module\Model;

use Magento\Framework\App\ObjectManager;

class Example
{
    public function doSomething(): void
    {
        $objectManager = ObjectManager::getInstance();
    }
}
```


## <a id="no-object-manager-type-hint"></a>`no-object-manager-type-hint`

Flags type hints that reference `Magento\Framework\ObjectManagerInterface` directly.
Injecting the ObjectManager into classes is considered an anti-pattern in Magento 2.
Instead, inject the specific interfaces or classes you need and let the DI container
handle the wiring.


### Requirements

- **Integration:** `Magento`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

namespace Vendor\Module\Model;

use Magento\Catalog\Api\ProductRepositoryInterface;

class Example
{
    public function __construct(
        private ProductRepositoryInterface $productRepository,
    ) {}
}
```

#### Incorrect code

```php
<?php

namespace Vendor\Module\Model;

use Magento\Framework\ObjectManagerInterface;

class Example
{
    public function __construct(
        private ObjectManagerInterface $objectManager,
    ) {}
}
```


## <a id="no-proxy-interceptor-in-constructor"></a>`no-proxy-interceptor-in-constructor`

Flags type hints that reference Proxy or Interceptor classes. In Magento 2,
Proxy and Interceptor classes are auto-generated by the framework. They should
be configured in `di.xml`, not referenced directly in PHP code.


### Requirements

- **Integration:** `Magento`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

namespace Vendor\Module\Model;

use Magento\Catalog\Api\ProductRepositoryInterface;

class Example
{
    public function __construct(
        private ProductRepositoryInterface $productRepository,
    ) {}
}
```

#### Incorrect code

```php
<?php

namespace Vendor\Module\Model;

use Magento\Catalog\Api\ProductRepositoryInterface\Proxy;

class Example
{
    public function __construct(
        private Proxy $productRepository,
    ) {}
}
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


## <a id="no-test-namespace-import"></a>`no-test-namespace-import`

Flags `use` statements that import classes from test namespaces.
Application modules should not depend on test classes in production code.


### Requirements

- **Integration:** `Magento`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

namespace Vendor\Module\Model;

use Magento\Catalog\Api\ProductRepositoryInterface;
```

#### Incorrect code

```php
<?php

namespace Vendor\Module\Model;

use Magento\TestFramework\Helper\Bootstrap;
```


## <a id="no-this-in-template"></a>`no-this-in-template`

Flags usage of `$this` in `.phtml` template files. In Magento 2, `$this` is
deprecated in templates since Magento 2.1. Use `$block` instead to access
block methods, or better yet, use a ViewModel.


### Requirements

- **Integration:** `Magento`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php
/** @var \Magento\Framework\View\Element\Template $block */
$block->getChildHtml('child');
```

#### Incorrect code

```php
<?php
// In a .phtml template:
$this->getChildHtml('child');
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

#### Incorrect code

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


## <a id="prefer-early-continue"></a>`prefer-early-continue`

Suggests using early continue pattern when a loop body contains only a single if statement.

This improves code readability by reducing nesting and making the control flow more explicit.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |
| `max_allowed_statements` | `integer` | `0` |

### Examples

#### Correct code

```php
<?php

for ($i = 0; $i < 10; $i++) {
    if (!$condition) {
        continue;
    }
    doSomething();
}
```

#### Incorrect code

```php
<?php

for ($i = 0; $i < 10; $i++) {
    if ($condition) {
        doSomething();
    }
}
```


## <a id="prefer-first-class-callable"></a>`prefer-first-class-callable`

Promotes the use of first-class callable syntax (`...`) for creating closures.

This rule identifies closures and arrow functions that do nothing but forward their arguments to another function or method.
In such cases, the more concise and modern first-class callable syntax, introduced in PHP 8.1, can be used instead.
This improves readability by reducing boilerplate code.

By default, this rule only checks method and static method calls. Optionally, function calls can also
be checked by enabling `check-functions`, but this may produce false positives with internal PHP
functions that enforce strict argument counts.


### Requirements

- **PHP version:** >= `8.1.0`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `check-functions` | `boolean` | `false` |

### Examples

#### Correct code

```php
<?php

$names = ['Alice', 'Bob', 'Charlie'];
$uppercased_names = array_map($formatter->format(...), $names);
```

#### Incorrect code

```php
<?php

$names = ['Alice', 'Bob', 'Charlie'];
$uppercased_names = array_map(fn($name) => $formatter->format($name), $names);
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


## <a id="prefer-static-closure"></a>`prefer-static-closure`

Suggests adding the `static` keyword to closures and arrow functions that don't use `$this`.

Static closures don't bind `$this`, making them more memory-efficient and their intent clearer.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"help"` |

### Examples

#### Correct code

```php
<?php

class Foo {
    public function bar() {
        // Static closure - doesn't use $this
        $fn = static fn($x) => $x * 2;

        // Non-static - uses $this
        $fn2 = fn() => $this->doSomething();

        // Static function - doesn't use $this
        $closure = static function($x) {
            return $x * 2;
        };
    }
}
```

#### Incorrect code

```php
<?php

class Foo {
    public function bar() {
        // Missing static - doesn't use $this
        $fn = fn($x) => $x * 2;

        // Missing static - doesn't use $this
        $closure = function($x) {
            return $x * 2;
        };
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


## <a id="require-namespace"></a>`require-namespace`

Detects files that contain definitions (classes, interfaces, enums, traits, functions, or constants)
but do not declare a namespace. Using namespaces helps avoid naming conflicts and improves code organization.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

namespace App;

class Foo {}
```

#### Incorrect code

```php
<?php

class Foo {}
```


## <a id="single-class-per-file"></a>`single-class-per-file`

Ensures that each file contains at most one class-like definition (class, interface, enum, or trait).



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

namespace App;

class Foo
{
}
```

#### Incorrect code

```php
<?php

namespace App;

class Foo
{
}

class Bar
{
}
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

