<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 */
final class GenPairSwap
{
    /**
     * @param A $first
     * @param B $second
     */
    public function __construct(public mixed $first, public mixed $second)
    {
    }

    /**
     * @return GenPairSwap<B, A>
     */
    public function swap(): GenPairSwap
    {
        return new GenPairSwap($this->second, $this->first);
    }
}

/**
 * @var GenPairSwap<int, string> $p
 *
 * @mago-expect analysis:redundant-docblock-type
 */
$p = new GenPairSwap(1, 'two');
$swapped = $p->swap();
echo $swapped->first . ' ' . $swapped->second;
