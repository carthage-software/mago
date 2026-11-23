<?php

declare(strict_types=1);

/**
 * @template T of list<int>|IteratorAggregate<int, string>
 *
 * @param T $values
 */
function foreach_template_union_bound($values): int|string
{
    foreach ($values as $value) {
        return $value;
    }

    return 0;
}

/**
 * @template U of list<string>
 * @template T of U
 *
 * @param T $values
 */
function foreach_template_bounded_by_template($values): string
{
    foreach ($values as $value) {
        return $value;
    }

    return '';
}
