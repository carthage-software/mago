<?php

declare(strict_types=1);

/**
 * @param array{name?: null|string} $a
 */
function flow_combined_isset_null(array $a): string
{
    if (isset($a['name'])) {
        return $a['name'];
    }

    return 'default';
}
