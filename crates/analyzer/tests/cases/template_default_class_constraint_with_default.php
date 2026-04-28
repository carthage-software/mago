<?php

declare(strict_types=1);

/**
 * @template T of int|string = int
 */
final class Holder
{
    /**
     * @param T $value
     */
    public function __construct(public readonly mixed $value) {}
}

/**
 * @param Holder $h
 *
 * @return int
 */
function read_default(Holder $h): int
{
    return $h->value;
}

echo read_default(new Holder(42));
