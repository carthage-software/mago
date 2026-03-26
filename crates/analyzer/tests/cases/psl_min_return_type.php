<?php

declare(strict_types=1);

/**
 * @template T of int|float
 *
 * @param list<T> $numbers
 *
 * @return T|null
 */
function test_min(array $numbers): null|int|float
{
    if ([] === $numbers) {
        return null;
    }

    return \min($numbers);
}
