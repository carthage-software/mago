<?php

declare(strict_types=1);

/**
 * @param 'a'|'b'|'c' $tag
 */
function flow_eq_literal_string(string $tag): int
{
    if ($tag === 'a') {
        return 1;
    }

    if ($tag === 'b') {
        return 2;
    }

    return 3;
}
