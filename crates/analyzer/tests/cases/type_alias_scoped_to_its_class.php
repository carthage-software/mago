<?php

declare(strict_types=1);

/**
 * @psalm-type Pair = array{0: int, 1: int}
 */
final class Data
{
    /** @param Pair $x */
    public function sum(array $x): int
    {
        return $x[0] + $x[1];
    }
}

/**
 * @psalm-import-type Pair from Data
 */
final class Other
{
    /** @param Pair $x */
    public function sum(array $x): int
    {
        return $x[0] + $x[1];
    }
}

// @mago-expect analysis:non-existent-class-like
/** @param Pair $x */
function after(mixed $x): mixed
{
    return $x;
}
