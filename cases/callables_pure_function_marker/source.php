<?php

declare(strict_types=1);

/**
 * @pure
 */
function callables_pure_add(int $a, int $b): int
{
    return $a + $b;
}

echo callables_pure_add(1, 2);
