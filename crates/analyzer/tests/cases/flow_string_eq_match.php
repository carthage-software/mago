<?php

declare(strict_types=1);

/**
 * @param 'red'|'green'|'blue' $color
 */
function flow_string_eq_match(string $color): int
{
    return match ($color) {
        'red' => 1,
        'green' => 2,
        'blue' => 3,
    };
}
