<?php

declare(strict_types=1);

/**
 * @param 1|2|3 $value
 */
function flow_match_no_default_int(int $value): string
{
    return match ($value) {
        1 => 'one',
        2 => 'two',
        3 => 'three',
    };
}
