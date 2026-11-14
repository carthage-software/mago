<?php

declare(strict_types=1);

/** @template T */
final class ThisHolder
{
    /** @param T $value */
    public function __construct(
        public mixed $value,
    ) {}
}

class ThisNode
{
    /** @return ThisHolder<static> */
    public function hold(): ThisHolder
    {
        return new ThisHolder($this);
    }
}
