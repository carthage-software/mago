<?php

abstract class Base {
    abstract public function getBase(): string;
}

interface Handler {
    public function handle(): void;
}

/**
 * @phpstan-require-extends Base
 * @phpstan-require-implements Handler
 */
trait CombinedTrait {
    public function getSelf(): self
    {
        return $this;
    }

    public function getBase(): string
    {
        return 'base';
    }

    public function handle(): void
    {
    }
}

class MyClass extends Base implements Handler
{
    use CombinedTrait;
}
