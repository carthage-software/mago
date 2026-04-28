<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenContainerW
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

/** @param GenContainerW<int> $g */
function take_int_g(GenContainerW $g): int
{
    return $g->get();
}

/** @param GenContainerW<int|string> $g */
function pass_widened(GenContainerW $g): void
{
    /** @mago-expect analysis:invalid-argument */
    take_int_g($g);
}
