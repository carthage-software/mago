<?php

declare(strict_types=1);

class A
{
    public function make(): static
    {
        return $this;
    }

    public function swap(A $other): static
    {
        // @mago-expect analysis:less-specific-return-statement
        return $other->make();
    }

    public function same(): static
    {
        return $this->make();
    }
}
