<?php

declare(strict_types=1);

/**
 * @return list<int>
 */
function offset_write_defines_possibly_undefined_variable(bool $condition): array
{
    if ($condition) {
        $values = [1];
    }

    $values[] = 2;

    return $values;
}

/**
 * @return array<string, string>
 */
function keyed_write_defines_possibly_undefined_variable(bool $condition): array
{
    if ($condition) {
        $values = ['a' => 'b'];
    }

    $values['c'] = 'd';

    return $values;
}
