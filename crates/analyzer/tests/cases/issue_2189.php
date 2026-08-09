<?php

declare(strict_types=1);

interface FooInterface
{
    public const int CST = 1;
}

/**
 * @require-implements FooInterface
 */
trait T
{
    public function getCst(): int
    {
        return self::CST;
    }
}

class Foo implements FooInterface
{
    use T;

    public const int CST = 2;
}
