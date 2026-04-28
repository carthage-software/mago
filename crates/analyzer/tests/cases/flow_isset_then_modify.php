<?php

declare(strict_types=1);

/**
 * @param array{count?: int} $a
 *
 * @return array{count: int}
 */
function flow_isset_then_modify(array $a): array
{
    if (!isset($a['count'])) {
        $a['count'] = 0;
    }

    $a['count']++;

    return $a;
}
