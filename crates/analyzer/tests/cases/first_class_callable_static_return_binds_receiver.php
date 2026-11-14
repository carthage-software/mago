<?php

declare(strict_types=1);

class Collection
{
    final public function __construct() {}

    public static function make(): static
    {
        return new static();
    }

    public function instance(): static
    {
        return $this;
    }
}

class Ints extends Collection
{
    public function sum(): int
    {
        return 0;
    }

    public function test(): int
    {
        $a = parent::make(...);
        $b = parent::instance(...);
        $c = $this->instance(...);
        $d = self::make(...);

        return $a()->sum() + $b()->sum() + $c()->sum() + $d()->sum();
    }
}

function outside(Ints $i): int
{
    $c = $i->instance(...);

    return $c()->sum();
}
