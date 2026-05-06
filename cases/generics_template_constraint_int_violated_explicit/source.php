<?php

declare(strict_types=1);

/**
 * @template T of int
 */
final class GenIntOnlyV
{
    /** @var T */
    public mixed $value;

    /** @param T $value */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }
}

new GenIntOnlyV('not an int');
