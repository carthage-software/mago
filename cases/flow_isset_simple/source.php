<?php

declare(strict_types=1);

/**
 * @param array{name?: string} $a
 */
function flow_isset_simple(array $a): string
{
    if (isset($a['name'])) {
        return $a['name'];
    }

    return '';
}
