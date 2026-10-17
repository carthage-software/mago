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
function test_required_nullable_key(array $arr): ?int
{
    if (!isset($arr['foo'])) {
        // BUG: Mago removes `foo` from the shape instead of narrowing it to null.
        accept_null($arr['foo']);

        return $arr['foo'];
    }

    return $arr['foo'];
}

/**
 * @param array{foo?: ?int, bar: string} $arr
 */
function test_optional_nullable_key(array $arr): ?int
{
    if (!isset($arr['foo'])) {
        // BUG: Mago deletes the key (definite absence) instead of retaining `foo?: null`.
        return $arr['foo'];
    }

    return $arr['foo'];
}
