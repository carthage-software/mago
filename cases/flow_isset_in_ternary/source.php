<?php

declare(strict_types=1);

/**
 * @param array{name?: string} $a
 */
function flow_isset_in_ternary(array $a): string
{
    return isset($a['name']) ? $a['name'] : 'default';
}
