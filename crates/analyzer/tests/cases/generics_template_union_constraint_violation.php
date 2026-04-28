<?php

declare(strict_types=1);

/**
 * @template T of int|string
 */
final class GenIntOrStr2
{
    /** @param T $value */
    public function __construct(public int|string $value)
    {
    }
}

/** @mago-expect analysis:template-constraint-violation,invalid-argument */
new GenIntOrStr2(true);
