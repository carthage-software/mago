<?php

declare(strict_types=1);

/**
 * @template T
 */
interface GenIfaceTopA
{
    /** @return T */
    public function get(): mixed;
}

/**
 * @template T
 *
 * @extends GenIfaceTopA<T>
 */
interface GenIfaceMidA extends GenIfaceTopA
{
}

/**
 * @implements GenIfaceMidA<int>
 */
final class GenLeafA implements GenIfaceMidA
{
    public function get(): int
    {
        return 1;
    }
}

function take_int_diamond(int $n): void
{
}

take_int_diamond((new GenLeafA())->get());
