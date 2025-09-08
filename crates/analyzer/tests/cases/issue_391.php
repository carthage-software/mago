<?php

declare(strict_types=1);

namespace Psl\Iter {
    /**
     * Returns true if the given iterable contains the key.
     *
     * @template Tk
     * @template Tv
     *
     * @param iterable<Tk, Tv> $iterable
     * @param Tk $key
     *
     * @mago-expect analysis:missing-return-statement
     */
    function contains_key(iterable $iterable, mixed $key): bool
    {
    }
}

/**
 * @param array<string, int> $test
 */
function x(array $test): void
{
    if (\array_key_exists('test', $test)) {
        echo $test['test'];
    }
}

/**
 * @param array<string, int> $test
 */
function y(array $test): void
{
    if (Psl\Iter\contains_key($test, 'test')) {
        echo $test['test'];
    }
}
