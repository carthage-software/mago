<?php

declare(strict_types=1);

function flow_match_int_with_default(int $code): string
{
    return match ($code) {
        1, 2, 3 => 'low',
        4, 5, 6 => 'mid',
        default => 'high',
    };
}
