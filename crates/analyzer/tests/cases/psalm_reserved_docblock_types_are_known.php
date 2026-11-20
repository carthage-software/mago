<?php

declare(strict_types=1);

namespace App\ReservedDocblockTypes;

/**
 * @param non-empty-scalar $scalar
 * @param callable-array $callableArray
 * @param callable-object $callableObject
 * @param stringable-object $stringable
 */
function reserved_docblock_types($scalar, $callableArray, $callableObject, $stringable): string
{
    $callableArray();
    $callableObject();

    return (string) $scalar . (string) $stringable;
}

function reserved_docblock_types_calls(): string
{
    return reserved_docblock_types('value', [new \ArrayObject(), 'count'], static fn(): int => 1, new \Exception());
}
