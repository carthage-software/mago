<?php

declare(strict_types=1);

/**
 * @param array<int, int|null> $values
 */
function reassigned_index(array $values, int $index): bool
{
    return $values[$index] !== null && ($index = $index + 1) > 0 && $values[$index] !== null;
}

/**
 * @param array<int, int|null> $values
 */
function by_reference_index(array $values, int $index): bool
{
    return $values[$index] !== null && shift($index) && $values[$index] !== null;
}

function shift(int &$index): bool
{
    $index++;

    return true;
}
