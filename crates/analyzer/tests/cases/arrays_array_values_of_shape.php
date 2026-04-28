<?php

declare(strict_types=1);

/**
 * @param array{a: int, b: int} $shape
 * @return list<int>
 */
function values_of(array $shape): array
{
    return array_values($shape);
}
