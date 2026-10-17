<?php

declare(strict_types=1);

/**
 * @param array{descr: ?string} $a
 */
function test_access_nullable_required_key(array $a): ?string
{
    if (!isset($a['descr'])) {
        return $a['descr'];
    }

    return $a['descr'];
}

/**
 * @param array{descr: ?string} $a
 */
function test_key_exists_on_nullable_required_key(array $a): bool
{
    if (!isset($a['descr'])) {
        if (array_key_exists('descr', $a)) {
            return true;
        }
    }

    return false;
}

/**
 * @param array{descr?: ?string} $a
 */
function test_key_exists_on_nullable_optional_key(array $a): bool
{
    if (!isset($a['descr'])) {
        if (array_key_exists('descr', $a)) {
            return true;
        }
    }

    return false;
}

/**
 * @param array{descr?: ?string} $a
 */
function test_access_after_array_key_exists_on_optional_nullable(array $a): ?string
{
    if (!isset($a['descr'])) {
        if (array_key_exists('descr', $a)) {
            return $a['descr'];
        }
    }

    return null;
}

/**
 * @param array{descr?: ?string} $a
 */
function test_unguarded_access_on_optional_nullable(array $a): ?string
{
    if (!isset($a['descr'])) {
        // @mago-expect analysis:possibly-undefined-string-array-index
        return $a['descr'];
    }

    return $a['descr'];
}

/**
 * @param array{descr?: string} $a
 */
function test_negated_isset_on_non_nullable_optional_removes_key(array $a): ?string
{
    if (!isset($a['descr'])) {
        // @mago-expect analysis:undefined-string-array-index
        return $a['descr'];
    }

    return $a['descr'];
}
