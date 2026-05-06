<?php

declare(strict_types=1);

/**
 * @param non-falsy-string $s
 */
function flow_redundant_truthy(string $s): string
{
    if ($s) {
        return $s;
    }

    return '';
}
