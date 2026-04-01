<?php

declare(strict_types=1);

// =============================================================================
// callable-string type parsing and basic usage
// =============================================================================

/** @param callable-string $fn */
function takes_callable_string(string $fn): void
{
    $fn();
}

/** @return callable-string */
function returns_callable_string(): string
{
    return 'strlen';
}

// callable-string is a subtype of string
function takes_string(string $_s): void {}

takes_string(returns_callable_string());

// callable-string is a subtype of callable
function takes_callable(callable $_c): void {}

takes_callable(returns_callable_string());

// callable-string is a subtype of non-empty-string
/** @param non-empty-string $_s */
function takes_non_empty_string(string $_s): void {}

takes_non_empty_string(returns_callable_string());

function test_function_exists_basic(string $str): void
{
    if (function_exists($str)) {
        // $str is narrowed to callable-string
        takes_callable_string($str);
        takes_callable($str);
        takes_non_empty_string($str);
        Closure::fromCallable($str);
    }
}

function test_function_exists_in_union(string|int $val): void
{
    if (is_string($val) && function_exists($val)) {
        takes_callable_string($val);
        Closure::fromCallable($val);
    }
}

function test_function_exists_with_early_return(string $str): ?Closure
{
    if (!function_exists($str)) {
        return null;
    }

    // After the guard, $str is callable-string
    return Closure::fromCallable($str);
}

function test_is_callable_string(string $str): void
{
    if (is_callable($str)) {
        takes_callable_string($str);
        takes_callable($str);
        Closure::fromCallable($str);
    }
}

function test_is_callable_string_or_callable(string|callable $str): ?Closure
{
    if (is_string($str)) {
        if (is_callable($str)) {
            return Closure::fromCallable($str);
        }

        return null;
    }

    return Closure::fromCallable($str);
}

/** @param class-string $class */
function test_is_callable_class_array(string $class, string $value): mixed
{
    $callable = [$class, 'fromString'];
    if (is_callable($callable)) {
        return $callable($value);
    }

    return null;
}

function test_from_callable_with_function_exists(string $fn): ?Closure
{
    if (function_exists($fn)) {
        return Closure::fromCallable($fn);
    }

    return null;
}

function test_from_callable_with_is_callable(string $fn): ?Closure
{
    if (is_callable($fn)) {
        return Closure::fromCallable($fn);
    }

    return null;
}

/**
 * @param callable-string $callback
 */
function invoke_callable_string(string $callback): mixed
{
    return $callback();
}

/**
 * @param list<callable-string> $callbacks
 */
function invoke_all(array $callbacks): void
{
    foreach ($callbacks as $cb) {
        $cb();
    }
}

function test_no_narrowing_without_check(string $str): void
{
    /** @mago-expect analysis:less-specific-nested-argument-type */
    Closure::fromCallable($str);
}

function test_callable_string_is_truthy(string $str): void
{
    if (function_exists($str)) {
        // callable-string is always truthy
        /** @mago-expect analysis:redundant-condition */
        if ($str) {
            echo 'always here';
        }

        // callable-string is always non-empty
        /** @mago-expect analysis:redundant-comparison */
        /** @mago-expect analysis:redundant-condition */
        if ($str !== '') {
            echo 'always here too';
        }
    }
}

function test_pass_to_call_user_func(string $fn): void
{
    if (function_exists($fn)) {
        call_user_func($fn);
    }
}

/** @param callable(string): string $transformer */
function apply_transform(callable $transformer, string $input): string
{
    return $transformer($input);
}

function test_pass_callable_string_to_typed_callable(string $fn): ?string
{
    if (function_exists($fn)) {
        /** @mago-expect analysis:less-specific-nested-argument-type */
        return apply_transform($fn, 'hello');
    }

    return null;
}

function identity_transformer(string $id): string
{
    return $id;
}

echo apply_transform(identity_transformer(...), 'hello');
echo apply_transform('identity_transformer', 'hello');

// =============================================================================
// Cased callable-string types in docblocks
// =============================================================================

/** @param lowercase-callable-string $fn */
function takes_lowercase_callable_string(string $fn): void
{
    $fn();
}

/** @param uppercase-callable-string $fn */
function takes_uppercase_callable_string(string $fn): void
{
    $fn();
}

// =============================================================================
// Subtype/supertype relationships between string variants
// =============================================================================

// callable-string is a subtype of string
function sub_callable_to_string(): void
{
    /** @var callable-string $cs */
    $cs = 'strlen';
    takes_string($cs);
}

// callable-string is a subtype of non-empty-string
function sub_callable_to_non_empty_string(): void
{
    /** @var callable-string $cs */
    $cs = 'strlen';
    takes_non_empty_string($cs);
}

// lowercase-callable-string is a subtype of lowercase-string
/** @param lowercase-string $_s */
function takes_lowercase_string(string $_s): void {}

function sub_lowercase_callable_to_lowercase(): void
{
    /** @var lowercase-callable-string $cs */
    $cs = 'strtolower';
    takes_lowercase_string($cs);
}

// uppercase-callable-string is a subtype of uppercase-string
/** @param uppercase-string $_s */
function takes_uppercase_string(string $_s): void {}

function sub_uppercase_callable_to_uppercase(): void
{
    /** @var uppercase-callable-string $cs */
    $cs = 'STRTOUPPER';
    takes_uppercase_string($cs);
}

// lowercase-callable-string is a subtype of callable-string
function sub_lowercase_callable_to_callable(): void
{
    /** @var lowercase-callable-string $cs */
    $cs = 'strtolower';
    takes_callable_string($cs);
    takes_callable($cs);
}

// uppercase-callable-string is a subtype of callable-string
function sub_uppercase_callable_to_callable(): void
{
    /** @var uppercase-callable-string $cs */
    $cs = 'STRTOUPPER';
    takes_callable_string($cs);
    takes_callable($cs);
}

// callable-string is a subtype of callable
function sub_callable_string_to_callable(): void
{
    /** @var callable-string $cs */
    $cs = 'strlen';
    takes_callable($cs);
}

// =============================================================================
// is_callable on lowercase-string narrows to lowercase-callable-string
// =============================================================================

/** @param lowercase-string $str */
function test_is_callable_lowercase_string(string $str): void
{
    if (is_callable($str)) {
        takes_lowercase_callable_string($str);
        takes_lowercase_string($str);
        takes_callable($str);
    }
}

/** @param uppercase-string $str */
function test_is_callable_uppercase_string(string $str): void
{
    if (is_callable($str)) {
        takes_uppercase_callable_string($str);
        takes_uppercase_string($str);
        takes_callable($str);
    }
}
