<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenCtorOnly
{
    /** @param T $value */
    public function __construct(mixed $value)
    {
        $string = print_r($value, true);
        echo $string;
    }
}

new GenCtorOnly('x');
