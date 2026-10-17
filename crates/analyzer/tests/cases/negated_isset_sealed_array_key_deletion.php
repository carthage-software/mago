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
function test_negated_isset_required_nullable_key(array $arr): ?int
{
    if (!isset($arr['foo'])) {
        accept_null($arr['foo']);

        return $arr['foo'];
    }

    return $arr['foo'];
}

/**
 * @param array{val: string|null} $arr
 */
function test_negated_isset_return(array $arr): ?string
{
    if (!isset($arr['val'])) {
        return $arr['val'];
    }

    return $arr['val'];
}

/**
 * @param list{?string, int} $list
 */
function test_negated_isset_list_known_element(array $list): ?string
{
    if (!isset($list[0])) {
        return $list[0];
    }

    return $list[0];
}

/**
 * @param array{foo?: ?int, bar: string} $arr
 */
function test_negated_isset_optional_key_removes_key(array $arr): ?int
{
    if (!isset($arr['foo'])) {
        // @mago-expect analysis:undefined-string-array-index
        return $arr['foo'];
    }

    return $arr['foo'];
}

/**
 * @param array{foo: ?int, bar: string} $arr
 */
function test_key_exists_after_negated_isset(array $arr): bool
{
    if (!isset($arr['foo'])) {
        return array_key_exists('foo', $arr);
    }

    return true;
}
