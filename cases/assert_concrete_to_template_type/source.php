<?php

/**
 * @template T of array-key
 */
interface TypeInterface
{
    /**
     * @return T
     */
    public function assert(mixed $value): mixed;
}

/**
 * @template T of array-key
 *
 * @param TypeInterface<T> $type
 * @param array-key $value
 *
 * @return T
 */
function narrow_to_t(TypeInterface $type, mixed $value): mixed
{
    return $type->assert($value);
}
