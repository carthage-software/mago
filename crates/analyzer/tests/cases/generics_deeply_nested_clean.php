<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenNB
{
    /**
     * @param T $v
     */
    public function __construct(public mixed $v)
    {
    }

    /** @return T */
    public function get(): mixed
    {
        return $this->v;
    }
}

/**
 * @param GenNB<GenNB<GenNB<int>>> $b
 */
function consume_deep(GenNB $b): int
{
    return $b->get()->get()->get();
}

consume_deep(new GenNB(new GenNB(new GenNB(42))));
