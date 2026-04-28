<?php

declare(strict_types=1);

/**
 * @param list<int>|string $value
 */
function flow_is_array_narrow(array|string $value): int
{
    if (is_array($value)) {
        return count($value);
    }

    return strlen($value);
}
