<?php

declare(strict_types=1);

/**
 * @param array{name?: string} $a
 */
function flow_redundant_isset(array $a): bool
{
    return isset($a['name']);
}
