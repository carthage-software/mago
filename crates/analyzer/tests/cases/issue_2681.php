<?php

declare(strict_types=1);

namespace Issue2681;

/** @return array{bar?: mixed} */
function produceArray(): array
{
    return [];
}

function consumeArray(array $_): void {}

$foo = produceArray();
assert(!array_key_exists('bar', $foo) || is_array($foo['bar']), 'foobar');
$baz = $foo['bar'] ?? [];
consumeArray($baz);

/** @param array{bar?: mixed} $foo */
function coalesce(array $foo): array
{
    assert(!array_key_exists('bar', $foo) || is_array($foo['bar']));

    return $foo['bar'] ?? [];
}

/** @param array{bar?: mixed} $foo */
function reversedCondition(array $foo): array
{
    assert(is_array($foo['bar'] ?? null) || !array_key_exists('bar', $foo));

    return $foo['bar'] ?? [];
}

/** @param array{bar?: mixed} $foo */
function issetGuard(array $foo): array
{
    assert(!isset($foo['bar']) || is_array($foo['bar']));

    return $foo['bar'] ?? [];
}

/** @param array{bar?: mixed} $foo */
function conditional(array $foo): array
{
    if (!array_key_exists('bar', $foo) || is_array($foo['bar'])) {
        return $foo['bar'] ?? [];
    }

    return [];
}

/** @param array{bar?: mixed} $foo */
function earlyReturn(array $foo): array
{
    if (array_key_exists('bar', $foo) && !is_array($foo['bar'])) {
        return [];
    }

    return $foo['bar'] ?? [];
}

/**
 * @param array{bar?: mixed} $foo
 * @return array{bar?: array}
 */
function parentShape(array $foo): array
{
    assert(!array_key_exists('bar', $foo) || is_array($foo['bar']));

    return $foo;
}

/** @param array{bar?: mixed} $foo */
function stringValue(array $foo): string
{
    assert(!array_key_exists('bar', $foo) || is_string($foo['bar']));

    return $foo['bar'] ?? '';
}

/** @param array{bar?: mixed} $foo */
function objectValue(array $foo): \stdClass
{
    assert(!array_key_exists('bar', $foo) || $foo['bar'] instanceof \stdClass);

    return $foo['bar'] ?? new \stdClass();
}

/** @param array{bar?: mixed} $foo */
function stillOptional(array $foo): void
{
    assert(!array_key_exists('bar', $foo) || is_array($foo['bar']));

    // @mago-expect analysis:possibly-undefined-string-array-index
    $value = $foo['bar'];
}

/** @param array{bar?: string|array} $foo */
function unrelatedFlag(array $foo, bool $flag): array
{
    assert(!$flag || is_array($foo['bar'] ?? null));

    // @mago-expect analysis:invalid-return-statement
    return $foo['bar'] ?? [];
}

/** @param array{bar?: mixed} $foo */
function mixedQuotes(array $foo): array
{
    assert(!array_key_exists("bar", $foo) || is_array($foo['bar']));

    return $foo['bar'] ?? [];
}

/** @param array{0?: mixed} $foo */
function numericStringKey(array $foo): array
{
    assert(!array_key_exists('0', $foo) || is_array($foo[0]));

    return $foo[0] ?? [];
}

/** @param array{bar?: mixed, other: string|array} $foo */
function differentKey(array $foo): array
{
    assert(!array_key_exists('bar', $foo) || is_array($foo['other']));

    // @mago-expect analysis:invalid-return-statement
    return $foo['other'];
}

/** @param array{a: array{b: string|array}, ...} $foo */
function nestedPathIsNotLiteralKey(array $foo): array
{
    assert(!array_key_exists("a']['b", $foo) || is_array($foo['a']['b']));

    // @mago-expect analysis:invalid-return-statement
    return $foo['a']['b'];
}
