<?php

declare(strict_types=1);

/**
 * @param null $val
 */
function accept_null($val): void
{
}

/**
 * @param array{foo: ?int, bar: string} $arr
 */
function test_required_nullable_key_narrowed_to_null(array $arr): ?int
{
    if (!isset($arr['foo'])) {
        accept_null($arr['foo']);

        return $arr['foo'];
    }

    return $arr['foo'];
}

/**
 * @param array{foo?: ?int, bar: string} $arr
 */
function test_optional_nullable_key_retains_null(array $arr): ?int
{
    if (!isset($arr['foo'])) {
        // @mago-expect analysis:possibly-undefined-string-array-index
        return $arr['foo'];
    }

    return null;
}

/**
 * @param array{foo?: ?int, bar: string} $arr
 */
function test_array_key_exists_guard_makes_read_safe(array $arr): ?int
{
    if (!isset($arr['foo'])) {
        if (array_key_exists('foo', $arr)) {
            accept_null($arr['foo']);

            return $arr['foo'];
        }
    }

    return null;
}

/**
 * @param array{foo?: ?int, bar: string} $arr
 */
function test_negated_array_key_exists_proves_absence(array $arr): ?int
{
    if (!array_key_exists('foo', $arr)) {
        // @mago-expect analysis:undefined-string-array-index
        return $arr['foo'];
    }

    return $arr['foo'];
}

/**
 * @param array{foo: ?int, bar: string} $arr
 */
function test_else_branch_retains_non_null(array $arr): void
{
    if (!isset($arr['foo'])) {
        accept_null($arr['foo']);

        return;
    }

    accept_null($arr['foo']); // @mago-expect analysis:invalid-argument
}
