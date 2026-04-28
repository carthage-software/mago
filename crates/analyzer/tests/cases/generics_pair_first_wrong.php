<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 */
final class GenPairFW
{
    /**
     * @param A $first
     * @param B $second
     */
    public function __construct(public mixed $first, public mixed $second)
    {
    }

    /** @return A */
    public function getFirst(): mixed
    {
        return $this->first;
    }
}

/** @param GenPairFW<int, string> $p */
function take_pair_first_str(GenPairFW $p): string
{
    /** @mago-expect analysis:invalid-return-statement */
    return $p->getFirst();
}
