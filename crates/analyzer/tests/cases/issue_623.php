<?php declare(strict_types=1);

/**
 * @return array{a: int}|array{b: int}|array{a: int, b: int, c: bool}
 */
function x(): array
{
    return [
        'a' => 1,
        'b' => 2,
        'c' => true,
    ];
}

/**
 * @mago-expect analysis:possibly-undefined-string-array-index
 */
function y(): void
{
    $x = x();
    if ($x['c'] === true) {
    }
}
