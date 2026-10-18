<?php

declare(strict_types=1);

/**
 * @param array{a: array<string, int>} $array
 * @return array{a: array{b: int, ...<string, int>}}
 * @throws LogicException
 */
function nestedGenericArray(array $array): array
{
    if (!isset($array['a']['b'])) {
        throw new LogicException();
    }

    return $array;
}

/**
 * @param array<string, array<string, string>> $array
 * @return array{foo: array{bar: string, ...<string, string>}, ...<string, array<string, string>>}|null
 */
function genericOuterArray(array $array): ?array
{
    if (isset($array['foo']['bar'])) {
        return $array;
    }

    return null;
}

/**
 * @param array{a: array{b: array<string, int>}} $array
 * @return array{a: array{b: array{c: int, ...<string, int>}}}|null
 */
function deeplyNestedArray(array $array): ?array
{
    if (isset($array['a']['b']['c'])) {
        return $array;
    }

    return null;
}

/**
 * @param array{a: list<string>} $array
 * @return array{a: list{string, ...<string>}}|null
 */
function nestedList(array $array): ?array
{
    if (isset($array['a'][0])) {
        return $array;
    }

    return null;
}

/**
 * @param array{a: array<string, int|null>} $array
 * @return array{a: array{b: int|null, ...<string, int|null>}}|null
 */
function nestedKeyExists(array $array): ?array
{
    if (array_key_exists('b', $array['a'])) {
        return $array;
    }

    return null;
}

/**
 * @param array{a: array<string, int>} $array
 * @return array{a: array{b: int, ...<string, int>}}
 */
function unguardedNestedArray(array $array): array
{
    // @mago-expect analysis:invalid-return-statement
    return $array;
}

/**
 * @param array{a: array<string, int>} $array
 * @return array{a: array{b: int, ...<string, int>}}|null
 */
function unrelatedKeyGuard(array $array): ?array
{
    if (isset($array['a']['other'])) {
        // @mago-expect analysis:invalid-return-statement
        return $array;
    }

    return null;
}
