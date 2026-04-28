<?php

declare(strict_types=1);

/**
 * @param 1|2|3 $value
 */
function flow_eq_literal_int(int $value): string
{
    if ($value === 1) {
        return 'one';
    }

    if ($value === 2) {
        return 'two';
    }

    return 'three';
}
