<?php

declare(strict_types=1);

/**
 * @param array{count?: int} $a
 */
function flow_isset_then_modify_value(array $a): int
{
    if (isset($a['count'])) {
        return $a['count'] * 2;
    }

    return 0;
}
