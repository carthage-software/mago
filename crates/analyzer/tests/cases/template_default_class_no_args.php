<?php

declare(strict_types=1);

/**
 * @template T = string
 */
final class Boxed
{
    /**
     * @param T $value
     */
    public function __construct(public readonly mixed $value) {}
}

/**
 * @param Boxed $b
 *
 * @return string
 */
function unwrap(Boxed $b): string
{
    return $b->value;
}

echo unwrap(new Boxed('hello'));
