<?php

declare(strict_types=1);

class Base
{
    /** @return $this */
    public function f(): static
    {
        return $this;
    }
}

class Child extends Base
{
    public function viaAncestorName(): static
    {
        return Base::f();
    }

    public function viaOwnName(): static
    {
        return Child::f();
    }

    public function viaKeywords(bool $b): static
    {
        return $b ? parent::f() : self::f();
    }
}
