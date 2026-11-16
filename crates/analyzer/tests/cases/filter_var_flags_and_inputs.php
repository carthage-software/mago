<?php

declare(strict_types=1);

function empty_string_null(string $value): string
{
    $filtered = filter_var($value, FILTER_DEFAULT, FILTER_FLAG_EMPTY_STRING_NULL);
    if ($filtered === null) {
        return 'empty';
    }

    return $filtered;
}

function default_filter(string $value): string
{
    return filter_var($value);
}

/**
 * @param array<int> $values
 */
function array_input(array $values): string
{
    $filtered = filter_var($values, FILTER_DEFAULT);
    if ($filtered === false) {
        return 'array';
    }

    return $filtered;
}

function stringable_input(Stringable $value): string
{
    return filter_var($value, FILTER_DEFAULT);
}

function scalar_input(int|float|bool|null $value): string
{
    return filter_var($value, FILTER_SANITIZE_SPECIAL_CHARS);
}
