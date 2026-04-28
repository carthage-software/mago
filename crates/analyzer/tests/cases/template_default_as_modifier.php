<?php

declare(strict_types=1);

/**
 * @template T as scalar = int
 */
final class Wrapper
{
    /**
     * @param T $value
     */
    public function __construct(public readonly mixed $value) {}
}

/**
 * @param Wrapper $w
 *
 * @return int
 */
function unwrap(Wrapper $w): int
{
    return $w->value;
}

echo unwrap(new Wrapper(42));
