<?php

declare(strict_types=1);

/** @template T */
final class GenericCallableBox
{
    /** @param T $value */
    public function __construct(private mixed $value) {}

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

/** @param GenericCallableBox<int> $box */
function generic_callable_from_callable(GenericCallableBox $box): int
{
    $closure = Closure::fromCallable([$box, 'get']);

    return $closure();
}

/** @param GenericCallableBox<int> $box */
function generic_callable_array(GenericCallableBox $box): int
{
    $callable = [$box, 'get'];

    return $callable();
}
