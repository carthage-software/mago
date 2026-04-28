<?php

declare(strict_types=1);

/**
 * @param array{name?: string} $a
 */
function flow_negated_isset(array $a): string
{
    if (!isset($a['name'])) {
        return 'absent';
    }

    return $a['name'];
}
