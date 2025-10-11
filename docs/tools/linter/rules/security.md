---
title: Security rules
outline: [2, 3]
---

# Security rules

This document details the rules available in the `Security` category.

| Rule | Code |
| :--- | :---------- |
| Disallowed Functions | [`disallowed-functions`](#disallowed-functions) |
| No Database Schema Changes | [`no-db-schema-change`](#no-db-schema-change) |
| No Debug Symbols | [`no-debug-symbols`](#no-debug-symbols) |
| No Insecure Comparison | [`no-insecure-comparison`](#no-insecure-comparison) |
| No Literal Password | [`no-literal-password`](#no-literal-password) |
| No Roles As Capabilities | [`no-roles-as-capabilities`](#no-roles-as-capabilities) |
| No Short Opening Tag | [`no-short-opening-tag`](#no-short-opening-tag) |
| No Unescaped Output | [`no-unescaped-output`](#no-unescaped-output) |
| Require `preg_quote` Delimiter | [`require-preg-quote-delimiter`](#require-preg-quote-delimiter) |
| Sensitive Parameter | [`sensitive-parameter`](#sensitive-parameter) |
| Tainted Data to Sink | [`tainted-data-to-sink`](#tainted-data-to-sink) |


## <a id="disallowed-functions"></a>`disallowed-functions`

Flags calls to functions that are disallowed via rule configuration.

You can specify which functions or extensions should be disallowed through the
`functions` or `extensions` options. This helps enforce coding standards,
security restrictions, or the usage of preferred alternatives.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |
| `functions` | `array` | `[]` |
| `extensions` | `array` | `[]` |

### Examples

#### Correct code

```php
<?php

function allowed_function(): void {
    // ...
}

allowed_function(); // Not flagged
```

#### Incorrect code

```php
<?php

curl_init(); // Error: part of a disallowed extension
```


## <a id="no-db-schema-change"></a>`no-db-schema-change`

This rule flags any attempt to alter the database schema (using `CREATE`, `ALTER`, or `DROP`)
within a `$wpdb` call. Schema modifications must only occur within a plugin activation hook
to prevent catastrophic performance issues and data corruption.


### Requirements

- **Integration:** `WordPress`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

function my_plugin_activate() {
    global $wpdb;

    // Running schema changes inside an activation hook is safe.
    $wpdb->query("ALTER TABLE {$wpdb->posts} ADD my_column VARCHAR(255)");
}

register_activation_hook(__FILE__, 'my_plugin_activate');
```

#### Incorrect code

```php
<?php

// This schema change runs on every page load, which is very dangerous.
global $wpdb;
$wpdb->query("ALTER TABLE {$wpdb->posts} ADD my_column VARCHAR(255)");
```


## <a id="no-debug-symbols"></a>`no-debug-symbols`

Flags calls to debug functions like `var_dump`, `print_r`, `dd`, etc.

These functions are useful for debugging, but they should not be committed to
version control as they can expose sensitive information and are generally not
intended for production environments.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"note"` |

### Examples

#### Correct code

```php
<?php

// Production-safe code
error_log('Processing user request.');
```

#### Incorrect code

```php
<?php

function process_request(array $data) {
    var_dump($data); // Debug call that should be removed
    // ...
}
```


## <a id="no-insecure-comparison"></a>`no-insecure-comparison`

Detects insecure comparison of passwords or tokens using `==`, `!=`, `===`, or `!==`.

These operators are vulnerable to timing attacks, which can expose sensitive information.
Instead, use `hash_equals` for comparing strings or `password_verify` for validating hashes.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

if (hash_equals($storedToken, $userToken)) {
    // Valid token
}
```

#### Incorrect code

```php
<?php

if ($storedToken == $userToken) {
    // Vulnerable to timing attacks
}
```


## <a id="no-literal-password"></a>`no-literal-password`

Detects the use of literal values for passwords or sensitive data.
Storing passwords or sensitive information as literals in code is a security risk
and should be avoided. Use environment variables or secure configuration management instead.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

$password = getenv('DB_PASSWORD');
```

#### Incorrect code

```php
<?php

$password = "supersecret";
```


## <a id="no-roles-as-capabilities"></a>`no-roles-as-capabilities`

This rule flags the use of user roles (e.g., `'administrator'`) in functions that expect a
granular capability (e.g., `'edit_posts'`). Checking against specific capabilities is a
core security principle in WordPress.


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

if ( current_user_can( 'edit_posts' ) ) { /* ... */ }
```

#### Incorrect code

```php
<?php

// This check is brittle and will fail if roles are customized.
if ( current_user_can( 'editor' ) ) { /* ... */ }
```


## <a id="no-short-opening-tag"></a>`no-short-opening-tag`

Disallows the use of short opening tags (`<?`).

The availability of `<?` depends on the `short_open_tag` directive in `php.ini`. If
this setting is disabled on a server, any code within the short tags will be
exposed as plain text, which is a significant security risk. Using the full `<?php`
opening tag is the only guaranteed portable way to ensure your code is always
interpreted correctly.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

echo "Hello, World!";
```

#### Incorrect code

```php
<?

echo "Hello, World!";
```


## <a id="no-unescaped-output"></a>`no-unescaped-output`

This rule ensures that any variable or function call that is output directly to the page is
properly escaped. All data must be escaped before printing to prevent Cross-Site Scripting (XSS)
vulnerabilities.


### Requirements

- **Integration:** `WordPress`

### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

echo esc_html( $user_comment );
?>
<a href="<?php echo esc_url( $user_provided_url ); ?>">Link</a>
```

#### Incorrect code

```php
<?php

// This is a major XSS vulnerability.
echo $_GET['user_comment'];
```


## <a id="require-preg-quote-delimiter"></a>`require-preg-quote-delimiter`

This rule requires that when using `preg_quote()`, the second `$delimiter` argument is always provided.
If the string being quoted contains the same character used for your regex delimiter (e.g., `/`),
failing to provide the second argument will prevent that character from being escaped,
which can break the regular expression.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"warning"` |

### Examples

#### Correct code

```php
<?php

// The delimiter is provided, ensuring it gets escaped if necessary.
$pattern = '/' . preg_quote( $user_input, '/' ) . '/';
```

#### Incorrect code

```php
<?php

// If $user_input contains '/', the regex will be invalid.
$pattern = '/' . preg_quote( $user_input ) . '/';
```


## <a id="sensitive-parameter"></a>`sensitive-parameter`

Requires that parameters that are likely to contain sensitive information (e.g., passwords)
are marked with the `#[SensitiveParameter]` attribute to prevent accidental logging or exposure.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `false` |
| `level` | `string` | `"error"` |

### Examples

#### Correct code

```php
<?php

function login(string $username, #[SensitiveParameter] string $password): void {
   // ...
}
```

#### Incorrect code

```php
<?php

function login(string $username, string $password): void {
   // ...
}
```


## <a id="tainted-data-to-sink"></a>`tainted-data-to-sink`

Detects user (tainted) data being passed directly to sink functions or constructs
(such as `echo`, `print`, or user-defined "log" functions). If these functions emit
or store data without sanitization, it could lead to Cross-Site Scripting (XSS)
or other injection attacks.



### Configuration

| Option | Type | Default |
| :--- | :--- | :--- |
| `enabled` | `boolean` | `true` |
| `level` | `string` | `"error"` |
| `known-sink-functions` | `array` | `["printf"]` |

### Examples

#### Correct code

```php
<?php

// Properly escape data before using a sink like `echo`
echo htmlspecialchars($_GET['name'] ?? '', ENT_QUOTES, 'UTF-8');
```

#### Incorrect code

```php
<?php

// This is considered unsafe:
echo $_GET['name'] ?? '';
```

