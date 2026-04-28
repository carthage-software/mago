<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 */
final class GenPairSW
{
    /**
     * @param A $first
     * @param B $second
     */
    public function __construct(public mixed $first, public mixed $second)
    {
    }

    /** @return B */
    public function getSecond(): mixed
    {
        return $this->second;
    }
}

/** @param GenPairSW<int, string> $p */
function take_pair_second_int(GenPairSW $p): int
{
    /** @mago-expect analysis:invalid-return-statement */
    return $p->getSecond();
}
