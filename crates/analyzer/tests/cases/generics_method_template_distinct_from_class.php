<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenDistinct
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }

    /**
     * @template U
     *
     * @param U $other
     *
     * @return U
     */
    public function getOther(mixed $other): mixed
    {
        return $other;
    }
}

function take_string_d(string $s): void
{
}

take_string_d((new GenDistinct(1))->getOther('hi'));
