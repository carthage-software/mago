<?php

declare(strict_types=1);

/** @return non-falsy-string */
function literal_and_int(int $id): string
{
    return "id-$id";
}

/** @return non-falsy-string */
function single_non_zero_character(string $suffix): string
{
    return "a$suffix";
}

/**
 * @param non-empty-string $first
 * @param non-empty-string $second
 *
 * @return non-falsy-string
 */
function two_non_empty_parts(string $first, string $second): string
{
    return "$first$second";
}

/**
 * @param non-falsy-string $truthy
 *
 * @return non-falsy-string
 */
function truthy_part(string $prefix, string $truthy): string
{
    return "$prefix$truthy";
}

/** @return non-falsy-string */
function two_ints(int $major, int $minor): string
{
    return "$major$minor";
}

/** @return non-falsy-string */
function zero_and_any_string(string $suffix): string
{
    // @mago-expect analysis:invalid-return-statement
    return "0$suffix";
}

/** @return non-falsy-string */
function escaped_zero_and_any_string(string $suffix): string
{
    // @mago-expect analysis:invalid-return-statement
    return "\x30$suffix";
}

/** @return non-falsy-string */
function any_string_and_zero(string $prefix): string
{
    // @mago-expect analysis:invalid-return-statement
    return "{$prefix}0";
}

/** @return non-falsy-string */
function single_int(int $value): string
{
    // @mago-expect analysis:invalid-return-statement
    return "$value";
}

/** @return non-empty-string */
function zero_prefix_is_non_empty(string $suffix): string
{
    return "0$suffix";
}
