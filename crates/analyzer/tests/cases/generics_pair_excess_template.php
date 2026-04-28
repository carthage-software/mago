<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 */
class GenPairXs
{
    /** @var A */
    public mixed $first;
    /** @var B */
    public mixed $second;

    /**
     * @param A $first
     * @param B $second
     */
    public function __construct(mixed $first, mixed $second)
    {
        $this->first = $first;
        $this->second = $second;
    }
}

/**
 * @mago-expect analysis:excess-template-parameter
 *
 * @extends GenPairXs<int, string, bool>
 */
final class GenPairXsChild extends GenPairXs
{
}
