<?php

/**
 * @return '/ABC/i'
 */
function test_sprintf_basic(): string
{
    return sprintf('/%s/i', 'ABC');
}

/**
 * @return 'Hello, World!'
 */
function test_sprintf_string_sub(): string
{
    return sprintf('Hello, %s!', 'World');
}

/**
 * @return 'Count: 42'
 */
function test_sprintf_int_sub(): string
{
    return sprintf('Count: %d', 42);
}

/**
 * @return '100%'
 */
function test_sprintf_percent_escape(): string
{
    return sprintf('%d%%', 100);
}

/**
 * @return 'Price: 3.140000'
 */
function test_sprintf_float_sub(): string
{
    return sprintf('Price: %f', 3.14);
}

/**
 * @return 'Pi: 3.14'
 */
function test_sprintf_float_precision(): string
{
    return sprintf('Pi: %.2f', 3.14);
}

/**
 * @return 'Hex: ff'
 */
function test_sprintf_hex(): string
{
    return sprintf('Hex: %x', 255);
}

/**
 * @return 'Oct: 17'
 */
function test_sprintf_octal(): string
{
    return sprintf('Oct: %o', 15);
}

/**
 * @return 'Bin: 1010'
 */
function test_sprintf_binary(): string
{
    return sprintf('Bin: %b', 10);
}

/**
 * @return '00042'
 */
function test_sprintf_zero_padded(): string
{
    return sprintf('%05d', 42);
}

/**
 * @return '+42'
 */
function test_sprintf_show_sign(): string
{
    return sprintf('%+d', 42);
}

/**
 * @return 'a=1 b=2'
 */
function test_sprintf_multiple_args(): string
{
    return sprintf('a=%d b=%d', 1, 2);
}

/** Non-literal format falls back to string */
function test_sprintf_non_literal(string $fmt): string
{
    return sprintf($fmt, 'x');
}

/**
 * @return truthy-string
 */
function test_sprintf_non_literal_arg_truthy(string $val): string
{
    return sprintf('Hello %s', $val);
}

/**
 * @return truthy-string
 */
function test_sprintf_non_literal_int_truthy(int $id): string
{
    return sprintf('id=%d', $id);
}

/**
 * @return non-empty-string
 */
function test_sprintf_non_literal_int_non_empty(int $n): string
{
    return sprintf('%d', $n);
}

/**
 * @return truthy-string
 */
function test_sprintf_non_literal_percent_truthy(int $rate): string
{
    return sprintf('rate: %d%%', $rate);
}

function test_sprintf_bare_string_specifier(string $val): string
{
    return sprintf('%s', $val);
}

/**
 * @return truthy-string
 */
function test_sprintf_string_with_width(string $val): string
{
    return sprintf('%10s', $val);
}
