<?php

declare(strict_types=1);

// `__toString` gets an implicit `: string` return type from the engine.
final class Money
{
    public function __construct(private int $cents)
    {
    }

    public function __toString()
    {
        return number_format($this->cents / 100, 2);
    }
}

// Docblock types without a native equivalent cannot satisfy the check.
final class LineWriter
{
    /** @var resource */
    public $handle;

    /** @var callable(string): string */
    public $formatter;
}
