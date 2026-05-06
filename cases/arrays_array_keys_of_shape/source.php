<?php

declare(strict_types=1);

/**
 * @param array{a: int, b: string} $shape
 * @return list<string>
 */
function keys_of(array $shape): array
{
    return array_keys($shape);
}
