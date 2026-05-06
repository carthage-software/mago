<?php

declare(strict_types=1);

/**
 * @return array<string, int>
 */
function keyed(): array
{
    return ['a' => 1, 'b' => 2];
}

/**
 * @return array{a: int, b: int}
 */
function shape(): array
{
    return ['a' => 1, 'b' => 2];
}
