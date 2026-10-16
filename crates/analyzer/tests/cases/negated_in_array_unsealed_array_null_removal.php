<?php

declare(strict_types=1);

function accept_int(int $_): void {}

/**
 * @param int|null $x
 * @param list<int|null> $y
 */
function test_unsound_negated_in_array(?int $x, array $y): void
{
    if (!in_array($x, $y, true)) {
        // @mago-expect analysis:possibly-null-argument
        accept_int($x);
    }
}

/**
 * @param int|null $x
 * @param list<int> $y
 */
function test_in_array_narrows_positive(?int $x, array $y): int
{
    if (in_array($x, $y, true)) {
        accept_int($x);
        return $x;
    }

    return 0;
}

/**
 * @param int|null $x
 */
function test_negated_in_array_sealed_tuple(?int $x): ?int
{
    $y = [1, 2];
    if (!in_array($x, $y, true)) {
        return $x;
    }

    accept_int($x);
    return $x;
}

/**
 * @param int|null $x
 * @param list<int|null> $y
 */
function test_negative_invalid_arg(?int $x, array $y): void
{
    if (!in_array($x, $y, true)) {
        // @mago-expect analysis:invalid-argument
        accept_int("not an int");
    }
}
