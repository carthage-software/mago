<?php declare(strict_types=1);

namespace Example;

/**
 * @template T of int|float
 *
 * @param T $start
 * @param T $end
 * @param T|null $step
 *
 * @return non-empty-list<T>
 */
function range(int|float $start, int|float $end, int|float|null $step = null): array
{
    return range($start, $end, $step);
}

range(start: 0.0, end: 1.0, step: 0.5);
