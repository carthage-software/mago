<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenNB2
{
    /** @param T $v */
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
 * @param GenNB2<GenNB2<int>> $b
 */
function take_deep(GenNB2 $b): void
{
}

/**
 * @param GenNB2<GenNB2<string>> $b
 */
function bridge_deep(GenNB2 $b): void
{
    /** @mago-expect analysis:invalid-argument */
    take_deep($b);
}
