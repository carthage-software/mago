<?php

declare(strict_types=1);

final class GenStaticFactory2
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

function takes_int_sf2(int $n): void
{
}

/** @mago-expect analysis:invalid-argument */
takes_int_sf2(GenStaticFactory2::wrap('hello'));
