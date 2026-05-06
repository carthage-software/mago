<?php

declare(strict_types=1);

/**
 * @param 'a'|'b'|'c' $k
 */
function pick(string $k): int
{
    return match ($k) {
        'a' => 1,
        'b' => 2,
        'c' => 3,
    };
}

echo pick('a');
