<?php

declare(strict_types=1);

/**
 * @param list<int> $items
 *
 * @return list<int>
 */
function double_each(array $items): array
{
    foreach ($items as &$v) {
        $v = $v * 2;
    }

    return $items;
}
