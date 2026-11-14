<?php

declare(strict_types=1);

/**
 * @template A
 *
 * @param A $a
 *
 * @return A
 *
 * @throws InvalidArgumentException
 */
function notEmpty(mixed $a): mixed
{
    if ($a === [] || $a === '') {
        throw new InvalidArgumentException('empty');
    }

    return $a;
}

/** @param iterable<int, string> $a */
function isEmpty(iterable $a): bool
{
    return $a === [] || $a instanceof EmptyIterator;
}

/**
 * @template T of array<int, string>
 *
 * @param T $a
 */
function zeroOrTwo(array $a): bool
{
    return $a === [] || count($a) === 2;
}
