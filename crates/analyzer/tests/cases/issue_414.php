<?php

abstract class Something
{
    final public static function create(): static
    {
        return static::from();
    }

    abstract protected static function from(): static;
}

final class Concrete extends Something
{
    #[Override]
    protected static function from(): static
    {
        return new Concrete();
    }
}

class Caller
{
    private function call(): Concrete
    {
        return Concrete::create();
    }
}
