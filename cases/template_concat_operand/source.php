<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 */
function unconstrained(mixed $value): string
{
    return $value . '!';
}

/**
 * @template T as mixed
 *
 * @param T $value
 */
function constrained_to_mixed(mixed $value): string
{
    return $value . '!';
}

/**
 * @template T as string
 *
 * @param T $value
 */
function constrained_to_string(string $value): string
{
    return $value . '!';
}

/**
 * @template T as int
 *
 * @param T $value
 */
function constrained_to_int(int $value): string
{
    return $value . '!';
}

/**
 * @template T as string|int
 *
 * @param T $value
 */
function constrained_to_string_or_int(string|int $value): string
{
    return $value . '!';
}

/**
 * @template T
 *
 * @param T $value
 *
 * @param-out T $value
 */
function compound_assignment(mixed &$value): void
{
    $value .= 'text';
}
