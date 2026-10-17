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
