<?php

declare(strict_types=1);

/**
 * @template T of string
 */
final class GenStrOnly
{
    /** @param T $value */
    public function __construct(public string $value)
    {
    }

    /** @return T */
    public function get(): string
    {
        return $this->value;
    }
}

function take_str(string $s): void
{
}

take_str((new GenStrOnly('hi'))->get());
