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

/** @mago-expect analysis:invalid-argument,template-constraint-violation */
new GenIntOnlyV('not an int');
