<?php

declare(strict_types=1);

interface FooInterface
{
    public const int CST = 1;

    /** @param positive-int $value */
    public function usePositive(int $value): void;
}

/** @param positive-int $value */
function takesPositive(int $value): void {}

/**
 * @require-implements FooInterface
 */
trait T
{
    public function getCst(): int
    {
        return self::CST;
    }

    /** @inheritDoc */
    public function usePositive(int $value): void
    {
        takesPositive($value);
    }
}

class Foo implements FooInterface
{
    use T;

    public const int CST = 2;
}
