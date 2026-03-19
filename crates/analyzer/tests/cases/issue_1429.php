<?php

declare(strict_types=1);

class Foo
{
    final public function __construct() {}

    /** @return $this */
    public function chained(): self
    {
        /** @mago-expect analysis:less-specific-return-statement */
        return new static();
    }

    /** @return $this */
    public function chained2(): self
    {
        /** @mago-expect analysis:less-specific-return-statement */
        return new self();
    }

    /** @return $this */
    public function chained3(): self
    {
        return $this; // fine
    }

    public static function create(): static
    {
        return new static(); // fine
    }

    public function create2(): static
    {
        return $this; // fine
    }

    public static function create3(): static
    {
        /** @mago-expect analysis:less-specific-return-statement */
        return new self();
    }
}
