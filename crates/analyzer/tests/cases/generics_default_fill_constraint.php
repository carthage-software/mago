<?php

declare(strict_types=1);

/**
 * @template T of int
 */
final class GenDefaultInt
{
    /** @var T */
    public mixed $value;

    /** @param T $value */
    public function __construct(mixed $value)
    {
        $this->value = $value;
    }

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

function take_int_default(int $n): void
{
}

$g = new GenDefaultInt(5);
take_int_default($g->get());
