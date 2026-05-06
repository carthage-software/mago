<?php

declare(strict_types=1);

/**
 * @template T of int|string = int
 */
final class Bag
{
    /**
     * @param T $value
     */
    public function __construct(
        public readonly mixed $value,
    ) {}
}

/**
 *
 * @param Bag<bool> $b
 */
function take(Bag $b): void
{
    echo (string) $b->value;
}
