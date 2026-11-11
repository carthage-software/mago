<?php

declare(strict_types=1);

namespace Issue2620;

/**
 * @param iterable<int, string> $it
 * @return array<int, string>
 */
function integerKeys(iterable $it): array
{
    if (!is_object($it)) {
        return $it;
    }

    return [];
}

/**
 * @param iterable<string, int> $it
 * @return array<string, int>
 */
function stringKeys(iterable $it): array
{
    if (is_object($it)) {
        return [];
    }

    return $it;
}

function plainIterable(iterable $it): array
{
    return !is_object($it) ? $it : [];
}

/** @param array<int, string> $_ */
function consumeArray(array $_): void {}

/** @param \Traversable<int, string> $_ */
function consumeObject(\Traversable $_): void {}

/** @param iterable<int, string> $it */
function bothBranches(iterable $it): void
{
    if (is_object($it)) {
        consumeObject($it);
    } else {
        consumeArray($it);
    }
}

/**
 * @param iterable<int, string>|null $it
 * @return array<int, string>|null
 */
function nullableIterable(?iterable $it): ?array
{
    return !is_object($it) ? $it : null;
}

/**
 * @template TKey of array-key
 * @template TValue
 * @param iterable<TKey, TValue> $it
 * @return array<TKey, TValue>
 */
function genericKeysAndValues(iterable $it): array
{
    return !is_object($it) ? $it : [];
}

/**
 * @template T of iterable<int, string>
 * @param T $it
 * @return array<int, string>
 */
function genericIterable(iterable $it): array
{
    return !is_object($it) ? $it : [];
}

/**
 * @param iterable<mixed, string> $it
 * @return array<array-key, string>
 */
function mixedKeys(iterable $it): array
{
    return !is_object($it) ? $it : [];
}

/**
 * @param iterable<int, string> $it
 * @return array<int, string>
 */
function differentObject(iterable $it, object $other): array
{
    if ($it !== $other) {
        // @mago-expect analysis:invalid-return-statement
        return $it;
    }

    return [];
}

/** @inheritors Excluded|Remaining */
abstract class Sealed {}

final class Excluded extends Sealed {}

final class Remaining extends Sealed {}

/**
 * @param iterable<int, string>|Sealed $it
 * @return array<int, string>
 */
function excludeNamedClasses(iterable|Sealed $it): array
{
    if ($it instanceof Excluded) {
        return [];
    }

    if ($it instanceof Remaining) {
        return [];
    }

    // @mago-expect analysis:invalid-return-statement
    return $it;
}
