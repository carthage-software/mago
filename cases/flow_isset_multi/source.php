<?php

declare(strict_types=1);

/**
 * @param array{name?: string, age?: int} $a
 */
function flow_isset_multi(array $a): string
{
    if (isset($a['name'], $a['age'])) {
        return $a['name'] . ':' . $a['age'];
    }

    return '';
}
