<?php

declare(strict_types=1);

/**
 * @template T
 */
interface GenIfaceA
{
    /** @return T */
    public function get(): mixed;
}

/**
 * @implements GenIfaceA<int>
 */
final class GenImplementsA implements GenIfaceA
{
    public function get(): int
    {
        return 7;
    }
}

function take_int_a(int $n): void
{
}

take_int_a((new GenImplementsA())->get());
