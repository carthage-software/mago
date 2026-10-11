<?php

declare(strict_types=1);

// A literal `null` default makes a native type nullable, as in PHP:
// `string $value = null` is `?string $value = null`.
function native_null_default(string $value = null): string
{
    if ($value === null) {
        return 'none';
    }

    return $value;
}

function native_null_default_used(int $value = null): int
{
    return $value ?? 0;
}

function native_qualified_null_default(string $value = \null): string
{
    return $value ?? 'none';
}

function native_parenthesized_null_default(string $value = (null)): string
{
    return $value ?? 'none';
}

function native_uppercase_null_default(string $value = NULL): string
{
    return $value ?? 'none';
}

function native_union_null_default(int|string $value = null): string
{
    return (string) ($value ?? 'none');
}

const NATIVE_NULL_DEFAULT_CONSTANT = null;

// PHP does not make the type nullable for a constant that holds `null`.
function native_constant_null_default(string $value = NATIVE_NULL_DEFAULT_CONSTANT): string
{
    // @mago-expect analysis:redundant-null-coalesce
    return $value ?? 'none';
}

function native_null_default_caller(): void
{
    echo native_null_default(null);
    echo native_null_default_used(null);
    echo native_uppercase_null_default(null);
    echo native_union_null_default(null);
}

function native_null_default_unchecked(string $value = null): int
{
    // @mago-expect analysis:possibly-null-argument
    return strlen($value);
}

/**
 * The docblock keeps the `null` of the native type.
 *
 * @param non-empty-string $value
 */
function native_null_default_with_docblock(string $value = null): string
{
    return $value ?? 'none';
}

/**
 * @param string|null $value
 */
function native_null_default_with_nullable_docblock(string $value = null): string
{
    return $value ?? 'none';
}

$closure = function (string $value = null): string {
    return $value ?? 'none';
};

$arrow = fn(int $value = null): int => $value ?? 0;

echo $closure(null);
echo $arrow(null);

interface NativeNullDefaultParent
{
    public function run(string $value = null): void;
}

final class NativeNullDefaultWidened implements NativeNullDefaultParent
{
    public function run(?string $value = null): void
    {
        echo $value ?? 'none';
    }
}

final class NativeNullDefaultNarrowed implements NativeNullDefaultParent
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function run(string $value = 'none'): void
    {
        echo $value;
    }
}
