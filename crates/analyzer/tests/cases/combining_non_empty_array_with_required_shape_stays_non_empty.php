<?php

declare(strict_types=1);

/**
 * @param non-empty-array<string, int> $values
 */
function take_non_empty(array $values): void
{
}

/**
 * @param non-empty-array<string, int> $plain
 * @param array{a: int, ...<string, int>} $shape
 */
function both_orders(array $plain, array $shape, bool $flag): void
{
    take_non_empty($flag ? $shape : $plain);
    take_non_empty($flag ? $plain : $shape);
}

/**
 * @param non-empty-array<string, int> $values
 */
function offset_write_in_branch(array $values, bool $flag): void
{
    if ($flag) {
        $values['a'] = 1;
    }

    take_non_empty($values);
}
