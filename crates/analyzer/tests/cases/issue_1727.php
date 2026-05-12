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

class Issue1727Transaction {}

class Issue1727Repository
{
    public function findOne(): ?Issue1727Transaction
    {
        return null;
    }
}

abstract class Issue1727Assert
{
    /**
     * @phpstan-assert !null $actual
     */
    final public static function assertNotNull(mixed $actual, string $message = ''): void
    {
    }
}

abstract class Issue1727TestCase extends Issue1727Assert
{
}

/**
 * @phpstan-require-extends Issue1727TestCase
 */
trait Issue1727InheritedAssertTrait
{
    protected function getRepo(): Issue1727Repository
    {
        return new Issue1727Repository();
    }

    protected function assertTransactionPresent(): void
    {
        $transaction = $this->getRepo()->findOne();

        $this->assertNotNull($transaction);
    }
}
