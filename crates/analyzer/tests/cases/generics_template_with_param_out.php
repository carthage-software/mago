<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 *
 * @return list<T>
 */
function gen_singleton(mixed $value): array
{
    return [$value];
}

/**
 * @param list<int> $a
 */
function take_list_of_int_s(array $a): void
{
    foreach ($a as $n) {
        echo $n;
    }
}

take_list_of_int_s(gen_singleton(42));
