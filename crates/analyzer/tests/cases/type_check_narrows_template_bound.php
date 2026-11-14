<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 */
function describe(mixed $value): string
{
    if (is_scalar($value)) {
        return (string) $value;
    }

    return 'complex';
}

/**
 * @template T of int|string|list<int>
 *
 * @param T $value
 */
function only_scalars(mixed $value): int|string
{
    if (is_scalar($value)) {
        return $value;
    }

    return 0;
}
