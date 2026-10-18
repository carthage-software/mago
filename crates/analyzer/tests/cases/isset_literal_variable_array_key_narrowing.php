<?php

declare(strict_types=1);

namespace Repro\IssetLiteralVariableArrayKeyNarrowing;

/**
 * @param array{foo: int} $_
 */
function accept_required_foo(array $_): void {}

/**
 * @param array{foo?: int} $arr
 * @param 'foo' $k
 */
function test_var_key(array $arr, string $k): void
{
    if (isset($arr[$k])) {
        accept_required_foo($arr);
    }
}

/**
 * @param array{foo?: int} $arr
 * @param 'foo' $k
 *
 * @throws \InvalidArgumentException
 */
function test_var_key_guard(array $arr, string $k): int
{
    if (!isset($arr[$k])) {
        throw new \InvalidArgumentException('Missing foo');
    }

    return $arr['foo'];
}

class Config
{
    public const KEY = 'foo';
}

/**
 * @param array{foo?: int} $arr
 *
 * @throws \InvalidArgumentException
 */
function test_const_key(array $arr): int
{
    if (!isset($arr[Config::KEY])) {
        throw new \InvalidArgumentException('Missing foo');
    }

    return $arr['foo'];
}

/**
 * @param array{foo?: int} $arr
 */
function test_isset_const_key(array $arr): void
{
    if (isset($arr[Config::KEY])) {
        accept_required_foo($arr);
    }
}

class SelfConfig
{
    public const KEY = 'foo';

    /**
     * @param array{foo?: int} $arr
     */
    public function test(array $arr): void
    {
        if (isset($arr[self::KEY])) {
            accept_required_foo($arr);
        }
    }
}

/**
 * @param array{foo?: int} $arr
 */
function test_assigned_var_key(array $arr): void
{
    $k = 'foo';
    if (isset($arr[$k])) {
        accept_required_foo($arr);
    }
}

/**
 * @param array{0: string} $_
 */
function accept_required_int_key(array $_): void {}

/**
 * @param array{0?: string} $arr
 * @param 0 $k
 */
function test_int_var_key(array $arr, int $k): void
{
    if (isset($arr[$k])) {
        accept_required_int_key($arr);
    }
}

class IntConfig
{
    public const ZERO = 0;
}

/**
 * @param array{0?: string} $arr
 *
 * @throws \InvalidArgumentException
 */
function test_int_const_key(array $arr): string
{
    if (!isset($arr[IntConfig::ZERO])) {
        throw new \InvalidArgumentException('Missing 0');
    }

    return $arr[0];
}

/** @param array{10?: int, 11?: int} $arr */
function test_post_increment_key(array $arr): void
{
    $key = 10;

    if (isset($arr[$key++])) {
        echo $arr[10];
        // @mago-expect analysis:possibly-undefined-int-array-index
        echo $arr[11];
    }
}

/** @param array{10?: int, 11?: int} $arr */
function test_post_decrement_key(array $arr): void
{
    $key = 11;

    if (isset($arr[$key--])) {
        echo $arr[11];
        // @mago-expect analysis:possibly-undefined-int-array-index
        echo $arr[10];
    }
}

class ParentConfig
{
    public const KEY = 'foo';
}

final class ChildConfig extends ParentConfig
{
    public const KEY = 'bar';

    /** @param array{foo?: int, bar?: int} $arr */
    public function test_parent_key(array $arr): void
    {
        if (isset($arr[parent::KEY])) {
            echo $arr['foo'];
            // @mago-expect analysis:possibly-undefined-string-array-index
            echo $arr['bar'];
        }
    }

    /** @param array{foo?: int, bar?: int} $arr */
    public function test_self_key(array $arr): void
    {
        if (isset($arr[self::KEY])) {
            echo $arr['bar'];
            // @mago-expect analysis:possibly-undefined-string-array-index
            echo $arr['foo'];
        }
    }
}

/** @param array{foo?: int|null} $arr */
function test_nullable_value(array $arr): int
{
    $key = 'foo';

    if (!isset($arr[$key])) {
        return 0;
    }

    return $arr['foo'];
}

/** @param array{outer?: array{foo?: int}} $arr */
function test_nested_keys(array $arr): int
{
    $outer = 'outer';
    $inner = 'foo';

    if (!isset($arr[$outer][$inner])) {
        return 0;
    }

    return $arr['outer']['foo'];
}

/**
 * @param array{foo?: int, bar?: int} $arr
 * @param 'foo'|'bar' $key
 */
function test_union_key(array $arr, string $key): void
{
    if (isset($arr[$key])) {
        echo $arr[$key];
        // @mago-expect analysis:possibly-undefined-string-array-index
        echo $arr['foo'];
        // @mago-expect analysis:possibly-undefined-string-array-index
        echo $arr['bar'];
    }
}

/** @param array{foo?: int, bar?: int} $arr */
function test_reassigned_key(array $arr): void
{
    $key = 'foo';

    if (isset($arr[$key])) {
        $key = 'bar';
        echo $arr['foo'];
        // @mago-expect analysis:possibly-undefined-string-array-index
        echo $arr[$key];
    }
}

/** @param array<string, int|null> $arr */
function test_quote_in_key(array $arr): int
{
    $key = "it's";

    if (isset($arr[$key])) {
        return $arr["it's"];
    }

    return 0;
}

/** @param array<string, int|null> $arr */
function test_backslash_in_key(array $arr): int
{
    $key = 'path\\';

    if (isset($arr[$key])) {
        return $arr['path\\'];
    }

    return 0;
}
