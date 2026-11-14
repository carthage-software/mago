<?php

declare(strict_types=1);

abstract class Factory
{
    final protected function addState(): static
    {
        return clone $this;
    }

    protected function addPlain(): static
    {
        return clone $this;
    }
}

class Child extends Factory
{
    public function approved(): static
    {
        return $this->addState();
    }

    public function approved_plain(): static
    {
        return $this->addPlain();
    }
}

final class GrandChild extends Child {}

class Maker
{
    final public function make(): static
    {
        // @mago-expect analysis:less-specific-return-statement
        return new self();
    }
}
