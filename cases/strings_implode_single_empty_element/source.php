<?php

declare(strict_types=1);

/**
 * @param list<string> $parts
 */
function implode_can_yield_empty(array $parts): bool
{
    $composite = implode(',', $parts);

    return $composite === '';
}
