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
    public function __construct(public readonly mixed $value) {}
}

/**
 * @mago-expect analysis:docblock-type-mismatch
 *
 * @param Bag<bool> $b
 */
function take(Bag $b): void
{
    echo (string) $b->value;
}
