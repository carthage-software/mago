<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 */
class GenPairMiss
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
 * @mago-expect analysis:missing-template-parameter
 *
 * @extends GenPairMiss<int>
 */
final class GenPairMissChild extends GenPairMiss
{
}
