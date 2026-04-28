<?php

class Issue1727Foo {}

class Issue1727Asserter
{
    /** @phpstan-assert !null $value */
    public function assertNotNullValue(mixed $value): void {}

    public function getFoo(): ?Issue1727Foo
    {
        return null;
    }
}

/**
 * @phpstan-require-extends Issue1727Asserter
 */
trait Issue1727Trait
{
    public function check(): void
    {
        $foo = $this->getFoo();

        $this->assertNotNullValue($foo);
    }
}
