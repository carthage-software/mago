<?php

declare(strict_types=1);

function isset_string_offset_keeps_string(string $value, int $position): string
{
    if (!isset($value[$position])) {
        return '';
    }

    return substr($value, $position);
}

function isset_string_offset_in_ternary(string $value, int $position): string
{
    return isset($value[$position]) ? substr($value, $position) : '';
}
