<?php

declare(strict_types=1);

/**
 * @template T of int|string
 */
final class Box
{
    /**
     * @param T $value
     */
    public function __construct(
        public int|string $value,
    ) {}

    public function same(): static
    {
        return $this;
    }
}

/**
 * @param Box<int> $box
 *
 * @return Box<int>
 */
function through_callable(Box $box): Box
{
    $same = $box->same(...);

    return $same();
}
