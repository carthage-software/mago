<?php

declare(strict_types=1);

/**
 * @param list<int>|string $v
 */
function flow_negated_is_array(array|string $v): int
{
    if (!is_array($v)) {
        return strlen($v);
    }

    return count($v);
}
