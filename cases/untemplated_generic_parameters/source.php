<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @api
 */
class Foo
{
    /**
     * @var T|null
     */
    public mixed $result = null;

    /**
     * @param (Closure(?T): void) $callback
     */
    public function invoke(Closure $callback): void
    {
        $callback($this->result);
    }
}

/**
 * @template T of array-key
 */
class ArrayKey
{
    /**
     * @param T $value
     */
    public function __construct(
        private readonly string|int $value,
    ) {}

    /**
     * @template U of array-key
     *
     * @param (Closure(T): U) $callback
     *
     * @return ArrayKey<U>
     */
    public function map(Closure $callback): ArrayKey
    {
        $result = $callback($this->value);

        return new ArrayKey($result);
    }

    /**
     * @template U of array-key
     *
     * @param (Closure(T, T): U) $callable
     *
     * @return ArrayKey<U>
     */
    public function compute(Closure $callable): ArrayKey
    {
        return $this->map(
            /**
             * @param T $value
             */
            static fn(string|int $value): string|int => $callable($value, $value),
        );
    }
}
