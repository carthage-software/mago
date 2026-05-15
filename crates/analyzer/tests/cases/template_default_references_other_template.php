<?php

declare(strict_types=1);

/**
 * @template T1
 * @template T2 = T1
 *
 * @param T1                    $value
 * @param null|callable(T1): T2 $transform
 *
 * @return T2
 */
function maybe_transform(mixed $value, ?callable $transform = null): mixed
{
    if (null === $transform) {
        return $value;
    }

    return $transform($value);
}

// T2 defaults to T1 = string; strtoupper must accept this without error.
echo strtoupper(maybe_transform('hello'));

// Explicit callable: T2 resolved from callable return type, not default.
echo strtoupper(maybe_transform('hello', fn(string $s): string => $s));
