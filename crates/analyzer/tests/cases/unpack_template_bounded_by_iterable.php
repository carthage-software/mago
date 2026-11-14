<?php

declare(strict_types=1);

function foo(int ...$numbers): int
{
    return count($numbers);
}

/**
 * @template T of Traversable<int, int>
 *
 * @param T $test
 */
function bar(Traversable $test): int
{
    return foo(...$test);
}

/**
 * @template T of list<int>
 *
 * @param T $test
 *
 * @return list<int>
 */
function baz(array $test): array
{
    foo(...$test);

    return [...$test];
}
