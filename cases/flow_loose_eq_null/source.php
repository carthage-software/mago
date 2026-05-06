<?php

declare(strict_types=1);

/**
 */
function flow_loose_eq_null(?int $v): int
{
    if ($v == null) {
        return -1;
    }

    return $v + 1;
}
