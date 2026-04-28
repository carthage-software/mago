<?php

declare(strict_types=1);

/**
 * @template T
 */
abstract class GenBaseSub
{
    /** @return T */
    abstract public function get(): mixed;
}

/**
 * @extends GenBaseSub<int>
 */
abstract class GenIntSub extends GenBaseSub
{
}

final class GenIntSubLeaf extends GenIntSub
{
    public function get(): int
    {
        return 1;
    }
}

function take_int_sub(int $n): void
{
}

take_int_sub((new GenIntSubLeaf())->get());
