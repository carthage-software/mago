<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 *
 * @return (T is null ? array{} : (T is array ? T : array{T}))
 */
function wrap($value): array
{
    if ($value === null) {
        return [];
    }

    return is_array($value) ? $value : [$value];
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return (T is not array ? array{T} : T)
 */
function wrap_negated($value): array
{
    return is_array($value) ? $value : [$value];
}

/**
 * @template T of int|string
 *
 * @param T $value
 *
 * @return (T is string ? T : int)
 */
function keep_string($value): string|int
{
    return $value;
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return (T is array ? T : string)
 */
function wrong_otherwise_branch($value): array // @mago-expect analysis:docblock-type-mismatch
{
    return [];
}
