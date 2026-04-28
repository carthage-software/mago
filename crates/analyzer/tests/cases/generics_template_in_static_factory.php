<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenFactory
{
    /** @param T $value */
    private function __construct(public mixed $value)
    {
    }

    /**
     * @template U
     *
     * @param U $value
     *
     * @return GenFactory<U>
     */
    public static function of(mixed $value): GenFactory
    {
        return new GenFactory($value);
    }
}

function take_int_factory(int $n): void
{
}

take_int_factory(GenFactory::of(7)->value);
