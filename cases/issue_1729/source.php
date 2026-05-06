<?php

/**
 * @param list<int> $items
 *
 * @throws \RuntimeException
 */
function issue_1729_test(array $items): void
{
    $total = 0;

    foreach ($items as $count) {
        if ($count > 10) {
            throw new \RuntimeException();
        }

        $total += $count;
    }

    if ($total > 100) {
        throw new \RuntimeException();
    }
}
