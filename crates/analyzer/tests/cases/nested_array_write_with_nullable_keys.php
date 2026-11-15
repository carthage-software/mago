<?php

declare(strict_types=1);

/** @return array<string, array<string, int>> */
function nested(?string $a, ?string $b): array
{
    $x = [];
    // @mago-expect analysis:possibly-null-array-index(2)
    $x[$a][$b] = 1;

    return $x;
}

/** @return array<string, array<string, array<string, int>>> */
function deeper(?string $a, ?string $b, ?string $c): array
{
    $x = [];
    // @mago-expect analysis:possibly-null-array-index(3)
    $x[$a][$b][$c] = 1;

    return $x;
}

/** @return array<string, array<int|string, int>> */
function nullable_int_key(string $a, ?int $b): array
{
    $x = [];
    // @mago-expect analysis:possibly-null-array-index
    $x[$a][$b] = 1;

    return $x;
}
