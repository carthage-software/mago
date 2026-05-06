<?php

declare(strict_types=1);

/**
 */
function flow_match_unhandled_thrown(string $s): int
{
    return match ($s) {
        'a' => 1,
        'b' => 2,
    };
}
