<?php

declare(strict_types=1);

abstract class AbstractRoot
{
    final public static function new(): static
    {
        exit(0);
    }
}

final class Alpha extends AbstractRoot
{
}

class Delta
{
    public null|Alpha $baseAlpha = null;
    public null|Alpha $alpha = null;
}

final class AlphaTest
{
    /**
     * @template ExpectedType
     * @param ExpectedType $expected
     * @assert =ExpectedType $actual
     */
    public static function assertSame(mixed $expected, mixed $actual): void
    {
        static::assertSame($actual, $expected);
    }

    public function run(): void
    {
        $delta = new Delta();
        $alpha = Alpha::new();

        self::assertSame($alpha, $delta->alpha);
        self::assertSame($alpha, $delta->baseAlpha);
    }
}
