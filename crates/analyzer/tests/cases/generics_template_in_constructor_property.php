<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenCtorProp
{
    /**
     * @param T $value
     */
    public function __construct(
        public readonly mixed $value,
    ) {
    }
}

$g = new GenCtorProp(42);
echo $g->value + 1;
