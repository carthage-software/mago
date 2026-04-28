<?php

declare(strict_types=1);

/**
 * @param array{user?: array{name?: string}} $a
 */
function flow_double_isset_short_circuit(array $a): string
{
    if (isset($a['user']) && isset($a['user']['name'])) {
        return $a['user']['name'];
    }

    return '';
}
