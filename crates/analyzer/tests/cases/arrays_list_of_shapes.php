<?php

declare(strict_types=1);

/**
 * @return list<array{id: int, name: string}>
 */
function rows(): array
{
    return [
        ['id' => 1, 'name' => 'a'],
        ['id' => 2, 'name' => 'b'],
    ];
}
