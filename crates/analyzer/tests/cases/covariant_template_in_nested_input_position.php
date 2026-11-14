<?php

declare(strict_types=1);

/**
 * @template-covariant T
 */
final class Box
{
    /**
     * @param T $value
     */
    public function __construct(
        private mixed $value,
    ) {}

    // @mago-expect analysis:invalid-template-parameter
    /**
     * @param list<T> $values
     */
    public function nested(array $values): bool
    {
        return in_array($this->value, $values, true);
    }

    // @mago-expect analysis:invalid-template-parameter
    /**
     * @param callable(): T $make
     */
    public function producer(callable $make): bool
    {
        return $make() === $this->value;
    }

    /**
     * @param callable(T): void $callback
     */
    public function consumer(callable $callback): void
    {
        $callback($this->value);
    }
}
