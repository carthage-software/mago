<?php

declare(strict_types=1);

function flow_match_returns_union(int $code): int|string
{
    return match ($code) {
        1, 2 => 'low',
        3, 4 => 5,
        default => 'other',
    };
}
