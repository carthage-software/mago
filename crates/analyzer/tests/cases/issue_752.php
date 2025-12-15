<?php declare(strict_types=1);

function foo(null|string $x, null|string $y): string
{
    assert(
        assertion: $x !== null || $y !== null,
        description: 'Either x or y must be provided (or both).',
    );

    $z = $x ?? $y;

    return $z;
}
