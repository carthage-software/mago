<?php

declare(strict_types=1);

final class GenStaticFactory
{
    /**
     * @template T
     *
     * @param T $value
     *
     * @return T
     */
    public static function wrap(mixed $value): mixed
    {
        return $value;
    }
}

function takes_int_sf(int $n): void
{
}

takes_int_sf(GenStaticFactory::wrap(5));
