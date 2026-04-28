<?php

declare(strict_types=1);

/**
 * @return array{a: int, ...}
 */
function unsealed_extra(): array
{
    return ['a' => 1, 'b' => 'two', 'c' => 3];
}
