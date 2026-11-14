<?php

declare(strict_types=1);

/**
 * @template A
 * @param A $a
 * @return A
 */
function identity(mixed $a): mixed
{
    return $a;
}

/**
 * @template U
 * @param (callable(int): U)|string $f
 * @return list<U>
 */
function apply(callable|string $f): array
{
    return is_string($f) ? [] : [$f(1)];
}

/** @param list<int> $xs */
function takesInts(array $xs): void
{
    echo count($xs);
}

takesInts(array_map(identity(...), [1, 2, 3]));
takesInts(array_map('identity', [1, 2, 3]));
takesInts(apply(identity(...)));
