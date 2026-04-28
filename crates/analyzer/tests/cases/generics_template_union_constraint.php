<?php

declare(strict_types=1);

/**
 * @template T of int|string
 */
final class GenIntOrStr
{
    /** @param T $value */
    public function __construct(public int|string $value)
    {
    }

    /** @return T */
    public function get(): int|string
    {
        return $this->value;
    }
}

function take_int_or_str(int|string $v): void
{
}

take_int_or_str((new GenIntOrStr(1))->get());
take_int_or_str((new GenIntOrStr('hi'))->get());
