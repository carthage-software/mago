<?php

declare(strict_types=1);

/**
 * @template T of object
 *
 * @param IteratorAggregate<T> $items
 *
 * @return T|null
 */
function first_of_aggregate(IteratorAggregate $items): ?object
{
    foreach ($items as $item) {
        return $item;
    }

    return null;
}

/**
 * @template T of object
 *
 * @param iterable<T> $items
 *
 * @return T|null
 */
function first_of_iterable(iterable $items): ?object
{
    foreach ($items as $item) {
        return $item;
    }

    return null;
}

/**
 * @template T
 *
 * @param Traversable<T> $items
 *
 * @return T|null
 */
function first_of_unbounded(Traversable $items): mixed
{
    foreach ($items as $item) {
        return $item;
    }

    return null;
}

class Entity
{
    public function id(): int
    {
        return 1;
    }
}

/**
 * @template T of Entity
 *
 * @implements IteratorAggregate<int, T>
 */
final class Collection implements IteratorAggregate
{
    /** @param list<T> $items */
    public function __construct(private array $items) {}

    public function getIterator(): Iterator
    {
        return new ArrayIterator($this->items);
    }

    /** @return T|null */
    public function first(): ?Entity
    {
        foreach ($this as $item) {
            return $item;
        }

        return null;
    }

    /**
     * @param Traversable<T> $others
     *
     * @return T|null
     */
    public function firstOf(Traversable $others): ?Entity
    {
        foreach ($others as $item) {
            return $item;
        }

        return null;
    }
}

/**
 * @template T of Entity
 *
 * @param Iterator<T> $items
 *
 * @return T|null
 */
function first_of_iterator(Iterator $items): ?Entity
{
    foreach ($items as $item) {
        return $item;
    }

    return null;
}

/**
 * @template T of Entity
 *
 * @param ArrayIterator<int, T> $items
 *
 * @return T|null
 */
function first_of_array_iterator(ArrayIterator $items): ?Entity
{
    foreach ($items as $item) {
        return $item;
    }

    return null;
}

/**
 * @template T of Entity
 *
 * @param Collection<T> $items
 *
 * @return T|null
 */
function first_of_collection(Collection $items): ?Entity
{
    foreach ($items as $item) {
        return $item;
    }

    return null;
}
