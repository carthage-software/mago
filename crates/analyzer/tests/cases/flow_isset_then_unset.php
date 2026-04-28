<?php

declare(strict_types=1);

/**
 * @param array{name?: string} $a
 */
function flow_isset_then_unset(array $a): string
{
    if (isset($a['name'])) {
        $name = $a['name'];
        unset($a['name']);
        return $name;
    }

    return '';
}
