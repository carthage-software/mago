<?php

declare(strict_types=1);

/**
 * @template T
 */
abstract class GenBaseChain
{
    /** @param T $val */
    public function __construct(public mixed $val)
    {
    }
}

/**
 * @template T
 *
 * @extends GenBaseChain<T>
 */
abstract class GenMidChain extends GenBaseChain
{
}

/**
 * @extends GenMidChain<int>
 */
final class GenLeafChainInt extends GenMidChain
{
}

function takes_int_chain(int $n): void
{
}

takes_int_chain((new GenLeafChainInt(1))->val);
