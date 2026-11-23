<?php

declare(strict_types=1);

/**
 * @return array{0: string, 1: string, 2: string, 3: string}
 */
function array_literal_next_key_follows_largest_key(): array
{
    return [
        1 => 'a',
        2 => 'b',
        0 => 'c',
        'd',
    ];
}
