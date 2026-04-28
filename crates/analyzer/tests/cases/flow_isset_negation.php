<?php

declare(strict_types=1);

/**
 * @param array{name?: string} $a
 */
function flow_isset_negation(array $a): string
{
    if (!isset($a['name'])) {
        return 'missing';
    }

    return $a['name'];
}
