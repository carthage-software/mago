<?php

declare(strict_types=1);

/**
 * @param scalar $value
 */
function isset_offset_on_scalar($value): ?string
{
    return isset($value[0]) ? $value : null;
}

/**
 * @param array-key $value
 */
function isset_offset_on_array_key($value): ?string
{
    return isset($value[0]) ? $value : null;
}

/**
 * @param numeric $value
 *
 * @return numeric-string|null
 */
function isset_offset_on_numeric($value): ?string
{
    return isset($value[0]) ? $value : null;
}

function isset_offset_on_object(object $value): object
{
    if (isset($value['key'])) {
        return $value;
    }

    return $value;
}
