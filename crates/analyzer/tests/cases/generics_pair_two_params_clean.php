<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 */
final class GenPair
{
    /**
     * @param A $first
     * @param B $second
     */
    public function __construct(public mixed $first, public mixed $second)
    {
    }

    /**
     * @return A
     */
    public function getFirst(): mixed
    {
        return $this->first;
    }

    /**
     * @return B
     */
    public function getSecond(): mixed
    {
        return $this->second;
    }
}

/**
 * @param GenPair<int, string> $pair
 */
function consume_pair(GenPair $pair): string
{
    return $pair->getSecond();
}

consume_pair(new GenPair(1, 'two'));
