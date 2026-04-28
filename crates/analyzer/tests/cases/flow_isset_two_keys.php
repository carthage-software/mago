<?php

declare(strict_types=1);

/**
 * @param array{name?: string, age?: int} $a
 */
function flow_isset_two_keys(array $a): string
{
    if (isset($a['name']) && isset($a['age'])) {
        return $a['name'] . ':' . $a['age'];
    }

    return '';
}
