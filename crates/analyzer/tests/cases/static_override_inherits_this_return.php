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
    #[Override]
    public function f(): static
    {
        return parent::f();
    }

    /** @return $this */
    public function viaThis(): static
    {
        return $this->f();
    }

    /** @return $this */
    public function viaSelf(): static
    {
        return self::f();
    }

    /** @return $this */
    public function viaStatic(): static
    {
        return static::f();
    }
}
