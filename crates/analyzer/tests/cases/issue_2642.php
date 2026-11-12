<?php

declare(strict_types=1);

namespace Issue2642;

function strings(string $p, string $a, string $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

function swapped(string $p, string $a, string $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($p === $b && $flag) {
        return 2;
    }

    return 3;
}

function elseifBranch(string $p, string $a, string $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    } elseif ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

function integers(int $p, int $a, int $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

function floats(float $p, float $a, float $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

function booleans(bool $p, bool $a, bool $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

final class Item {}

function objects(Item $p, Item $a, Item $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

function arrays(array $p, array $a, array $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

/**
 * @param list{string} $p
 * @param list{string} $a
 * @param list{string} $b
 */
function lists(array $p, array $a, array $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

/**
 * @param array{a: 1, b: 2} $p
 * @param array{a: 1, b: 2} $a
 * @param array{a: 1, b: 2} $b
 */
function arrayOrder(array $p, array $a, array $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

function insideInequality(string $p, string $a, string $b, bool $flag): int
{
    if ($p !== $a) {
        if ($flag && $p === $b) {
            return 2;
        }
    }

    return 3;
}

function laterInequality(string $p, string $a, string $b, bool $flag): int
{
    if ($p !== $a) {
        return 1;
    }
    if ($flag && $p !== $b) {
        return 2;
    }

    return 3;
}

function looseEquality(string $p, string $a, string $b, bool $flag): int
{
    if ($p == $a) {
        return 1;
    }
    if ($flag && $p == $b) {
        return 2;
    }

    return 3;
}

function noConjunction(string $p, string $a, string $b): int
{
    if ($p === $a) {
        return 1;
    }
    if ($p === $b) {
        return 2;
    }

    return 3;
}

function disjunction(string $p, string $a, string $b, bool $flag): int
{
    if ($p === $a) {
        return 1;
    }
    if ($flag || $p === $b) {
        return 2;
    }

    return 3;
}

function invertedGuard(string $p, string $a, string $b, bool $flag): int
{
    if ($p !== $a) {
        return 1;
    }
    if ($flag && $p === $b) {
        return 2;
    }

    return 3;
}

function narrowEquality(mixed $value, string $other): string
{
    if ($value === $other) {
        return $value;
    }

    return '';
}

function narrowInequalityGuard(mixed $value, string $other): string
{
    if ($value !== $other) {
        return '';
    }

    return $value;
}

/**
 * @param 1|2 $value
 * @return 2
 */
function literalNarrowing(int $value): int
{
    if ($value === 1) {
        return 2;
    }

    return $value;
}

function literalContradiction(int $value, bool $flag): int
{
    if ($value === 1) {
        return 1;
    }
    // @mago-expect analysis:redundant-comparison,redundant-logical-operation,impossible-condition
    if ($flag && $value === 1) {
        return 2;
    }

    return 3;
}
