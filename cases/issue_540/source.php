<?php

declare(strict_types=1);

function takesInt(int $value, int ...$values): int
{
    return $value + array_sum($values);
}

/**
 * @param array<int> $items
 */
function processItemsUsingTernaryCondition(array $items): int
{
    return count($items) ? takesInt(...$items) : 0;
}

/**
 * @param array<int> $items
 */
function processItemsUsingIf(array $items): int
{
    if (count($items) > 1) {
        return takesInt(...$items);
    } else {
        return 0;
    }
}

/**
 * @param array<int> $items
 */
function processItemsUsingTernaryCompare(array $items): int
{
    return count($items) > 1 ? takesInt(...$items) : 0;
}
