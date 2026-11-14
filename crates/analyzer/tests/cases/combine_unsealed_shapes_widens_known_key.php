<?php

declare(strict_types=1);

/**
 * @param array{age: int, ...<array-key, mixed>} $x
 * @param array{name: string, ...<array-key, mixed>} $y
 */
function age(array $x, array $y, bool $first): int
{
    $z = $first ? $x : $y;

    // @mago-expect analysis:mixed-return-statement
    return $z['age'] ?? 0;
}

/**
 * @param array{age: int, ...<int, string>} $x
 * @param array{name: string, ...<int, string>} $y
 */
function age_with_int_rest(array $x, array $y, bool $first): int
{
    $z = $first ? $x : $y;

    return $z['age'] ?? 0;
}
