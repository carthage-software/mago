<?php

declare(strict_types=1);

/**
 * @return array{id?: int}
 */
function get_arr(): array
{
    return [];
}

function use_int(int $int): void
{
    echo 'value = ' . (string) $int;
}

$foo = get_arr();

use_int($foo['id'] ?? 1);
use_int($foo['id'] ?? throw new RuntimeException('Well, shit...'));
use_int($foo['id'] ?? die());
