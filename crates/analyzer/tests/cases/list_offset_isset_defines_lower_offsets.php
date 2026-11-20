<?php

declare(strict_types=1);

/**
 * @param list{0?: string, 1?: string, 2?: string} $parts
 */
function list_isset_defines_lower_offsets(array $parts): string
{
    if (isset($parts[2])) {
        return $parts[0] . $parts[1] . $parts[2];
    }

    return '';
}

/**
 * @param list{0?: string, 1?: string, 2?: string} $parts
 */
function list_key_exists_defines_lower_offsets(array $parts): string
{
    if (array_key_exists(1, $parts)) {
        return $parts[0] . $parts[1];
    }

    return '';
}
