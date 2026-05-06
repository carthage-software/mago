<?php

/**
 * @template T
 * @template Ts
 *
 * @param iterable<T> $items
 * @param Closure(Ts, T): Ts $operation
 * @param Ts $initial
 *
 * @return Ts
 */
function reduce(iterable $items, Closure $operation, mixed $initial): mixed
{
    $result = $initial;
    foreach ($items as $item) {
        $result = $operation($result, $item);
    }
    return $result;
}

$result = reduce([1, 2, 3], fn(int $ac, int $curr): int => $ac + $curr, 0);
if ($result === 0) {
    echo 'zero';
} else {
    echo 'non-zero';
}
