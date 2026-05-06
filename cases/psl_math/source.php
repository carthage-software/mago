<?php

declare(strict_types=1);

namespace Psl\Math {
    /**
     * @template T of int|float
     *
     * @param list<T> $numbers
     *
     * @return ($numbers is non-empty-list<T> ? T : T|null)
     *
     * @pure
     */
    function max(array $numbers): null|int|float
    {
        return max($numbers);
    }

    /**
     * @template T of int|float
     *
     * @param T $first
     * @param T $second
     * @param T ...$rest
     *
     * @return T
     *
     * @pure
     */
    function maxva(int|float $first, int|float $second, int|float ...$rest): int|float
    {
        return maxva($first, $second, ...$rest);
    }

    /**
     * @template T of int|float
     *
     * @param list<T> $numbers
     *
     * @return ($numbers is non-empty-list<T> ? T : null)
     *
     * @pure
     */
    function min(array $numbers): null|float|int
    {
        return min($numbers);
    }

    /**
     * @template T of int|float
     *
     * @param T $first
     * @param T $second
     * @param T ...$rest
     *
     * @return T
     *
     * @pure
     */
    function minva(int|float $first, int|float $second, int|float ...$rest): int|float
    {
        return minva($first, $second, ...$rest);
    }

    /**
     * @template T of int|float
     *
     * @param T $number
     *
     * @return T
     *
     * @pure
     */
    function abs(int|float $number): int|float
    {
        return abs($number);
    }
}

namespace App {
    /**
     * @param non-negative-int $value
     */
    function takes_non_negative_int(int $value): void
    {
        takes_non_negative_int($value);
    }

    /**
     * @param positive-int $value
     */
    function takes_positive_int(int $value): void
    {
        takes_positive_int($value);
    }

    /**
     * @param non-positive-int $value
     */
    function takes_non_positive_int(int $value): void
    {
        takes_non_positive_int($value);
    }

    takes_non_negative_int(\Psl\Math\maxva(0, mt_rand(0, 255)));

    takes_positive_int(\Psl\Math\maxva(mt_rand(-100, 100), 1));

    takes_non_positive_int(\Psl\Math\minva(mt_rand(-100, 100), 0));

    takes_non_negative_int(\Psl\Math\abs(mt_rand(-100, 100)));
}
