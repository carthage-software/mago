<?php

declare(strict_types=1);

/**
 * @return array{a: int, b: string}
 */
function extra_key(): array
{
    return ['a' => 1, 'b' => 'two', 'c' => 'three'];
}
