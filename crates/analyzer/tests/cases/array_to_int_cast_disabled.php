<?php

declare(strict_types=1);

function arrayCast(array $values): int
{
    return (int) $values;
}

function integerCast(array $values): int
{
    return (integer) $values;
}

function unionCast(array|int $values): int
{
    return (int) $values;
}

/** @return 1 */
function nonEmptyCast(): int
{
    return (int) [10, 20];
}
