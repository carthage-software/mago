<?php

declare(strict_types=1);

/**
 * @return array<string, int>
 */
function x(): array
{
    $a = ['a' => 100, 'b' => 200, 'c' => 300, 'd' => 400];
    $b = array_slice(array: $a, offset: 0, length: 3);
    return $b;
}
