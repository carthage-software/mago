<?php

declare(strict_types=1);

/**
 * @param list<int|string> $values
 *
 * @return list<string>
 */
function numeric_strings(array $values): array
{
    $strings = [];
    foreach (array_filter($values, 'is_numeric') as $v) {
        if (is_string($v)) {
            $strings[] = $v;
        }
    }

    return $strings;
}

/** @psalm-assert-if-true int|string $value */
function is_int_or_string(mixed $value): bool
{
    return is_int($value) || is_string($value);
}

/**
 * @param list<int|string|null> $values
 *
 * @return list<string>
 */
function strings(array $values): array
{
    $strings = [];
    foreach (array_filter($values, 'is_int_or_string') as $v) {
        if (is_string($v)) {
            $strings[] = $v;
        }
    }

    return $strings;
}
