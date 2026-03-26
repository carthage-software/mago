<?php

function format_string_or_int(string|int $element): string
{
    if (is_int($element)) {
        return (string) $element;
    } else {
        return '\'' . $element . '\'';
    }
}

function format_string_or_bool(string|bool $element): string
{
    if (is_bool($element)) {
        return $element ? 'true' : 'false';
    } else {
        return '\'' . $element . '\'';
    }
}

function format_int_or_float(int|float $element): string
{
    if (is_int($element)) {
        return (string) $element;
    } else {
        return (string) $element;
    }
}

function format_string_or_int_or_bool(string|int|bool $element): string
{
    if (is_string($element)) {
        return '\'' . $element . '\'';
    } elseif (is_int($element)) {
        return (string) $element;
    } else {
        return $element ? 'true' : 'false';
    }
}

function format_string_or_float_or_bool(string|float|bool $element): string
{
    if (is_string($element)) {
        return '\'' . $element . '\'';
    } elseif (is_float($element)) {
        return (string) $element;
    } else {
        return $element ? 'true' : 'false';
    }
}

function format_any(string|int|float|bool $element): string
{
    if (is_string($element)) {
        return '\'' . $element . '\'';
    } elseif (is_int($element)) {
        return (string) $element;
    } elseif (is_float($element)) {
        return (string) $element;
    } else {
        return $element ? 'true' : 'false';
    }
}

function narrow_int_or_float_to_float(int|float $value): float
{
    if (is_float($value)) {
        return $value;
    } else {
        return 0.0;
    }
}

function narrow_int_or_float_to_int(int|float $value): int
{
    if (is_float($value)) {
        return 0;
    } else {
        return $value;
    }
}
