<?php

declare(strict_types=1);

/**
 * @param mixed|false $value
 */
function type_checks_narrow_mixed_part_of_union(mixed $value): string
{
    if (is_resource($value)) {
        return (string) stream_get_contents($value);
    }

    if (is_object($value)) {
        return $value::class;
    }

    if (is_array($value)) {
        return (string) count($value);
    }

    return '';
}
