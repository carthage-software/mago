<?php

declare(strict_types=1);

final class ClosureThisContainer
{
    public int $value = 42;

    public function getValue(): int
    {
        return $this->value;
    }
}

/**
 * @param-closure-this ClosureThisContainer $callback
 */
function with_container_scope(Closure $callback): void
{
    $callback();
}

with_container_scope(function (): void {
    echo $this->value;
    echo $this->getValue();
});

with_container_scope(function (): void {
    // @mago-expect analysis:non-existent-property
    echo $this->missing;
});

// Arrow functions passed directly are bound the same way.
with_container_scope(fn() => $this->value + $this->getValue());

/**
 * @template T of object
 */
trait ClosureThisExtendable
{
    /**
     * @param-closure-this T $callback
     */
    public function register(Closure $callback): void
    {
        $callback();
    }
}

/**
 * @template TValue
 */
final class ClosureThisExpectation
{
    public int $value = 1;

    /** @use ClosureThisExtendable<self<TValue>> */
    use ClosureThisExtendable;
}

function exercise_closure_this_template(ClosureThisExpectation $expectation): void
{
    // `T` resolves to `self<TValue>`, i.e. ClosureThisExpectation, via the `@use` clause.
    $expectation->register(function (): void {
        echo $this->value;
    });
}
