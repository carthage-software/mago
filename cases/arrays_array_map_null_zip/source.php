<?php

declare(strict_types=1);

/**
 * @param list<int> $a
 * @param list<int> $b
 */
function consume_zip(array $a, array $b): void
{
    foreach (array_map(null, $a, $b) as $pair) {
        echo (string) ($pair[0] ?? 0);
    }
}
