<?php

declare(strict_types=1);

// Regression for a comparator bug: passing `iterable<Y>` to a parameter typed
// as `Traversable<mixed, mixed>|array<array-key, mixed>` was rejected, even
// though `iterable<T>` is defined as `Traversable<mixed, T>|array<array-key, T>`.
//
// The mirror direction (passing `Traversable<mixed, Y>|array<array-key, Y>`
// into an `iterable<mixed>` parameter) worked, so the asymmetry pointed at the
// type-comparator rather than at type-resolution.

class Y {}

/**
 * @return iterable<Y>
 *
 * @pure
 */
function x(): iterable
{
    return [];
}

/**
 * @param Traversable<mixed, mixed>|array<array-key, mixed> $y
 *
 * @return int<0, max>
 */
function z(Traversable|array $y): int
{
    $i = 0;
    foreach ($y as $_) {
        $i++;
    }

    return $i;
}

// `iterable<Y>` ≡ `Traversable<mixed, Y>|array<array-key, Y>`, which is
// compatible with `Traversable<mixed, mixed>|array<array-key, mixed>`.
$n = z(x());

// And the reverse: `Traversable<mixed, Y>|array<array-key, Y>` passed into
// `iterable<mixed>` should also hold.
/**
 * @return Traversable<mixed, Y>|array<array-key, Y>
 *
 * @pure
 */
function x2(): Traversable|array
{
    return [];
}

/**
 * @param iterable<mixed> $y
 *
 * @return int<0, max>
 */
function z2(iterable $y): int
{
    $i = 0;
    foreach ($y as $_) {
        $i++;
    }

    return $i;
}

$m = z2(x2());

var_dump($n, $m);
