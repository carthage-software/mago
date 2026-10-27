<?php

declare(strict_types=1);

class Entity
{
}

/**
 * @template T of Entity
 * @implements IteratorAggregate<int, T>
 */
final class Collection implements IteratorAggregate
{
    /** @param list<T> $items */
    public function __construct(private array $items)
    {
    }

    public function getIterator(): Iterator
    {
        return new ArrayIterator($this->items);
    }

    /**
     * @return T|null
     * @throws Exception
     */
    public function first(): ?Entity
    {
        foreach ($this as $item) {
            return $item;
        }

        return null;
    }
}

/**
 * @template T of Entity
 * @param Collection<T> $items
 * @return T|null
 * @throws Exception
 */
function first_of_collection(Collection $items): ?Entity
{
    foreach ($items as $item) {
        return $item;
    }

    return null;
}
