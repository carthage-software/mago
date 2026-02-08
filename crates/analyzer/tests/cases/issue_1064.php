<?php

declare(strict_types=1);

class Foo
{
    const int FOO = 42;
}

/**
 * @psalm-require-extends Foo
 */
trait Bar
{
    public function foo(): int
    {
        return static::FOO;
    }
}
