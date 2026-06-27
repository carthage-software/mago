<?php

declare(strict_types=1);

/**
 * @template T
 */
final class TypedList
{
    public function __construct(
        /** @param list<T> */
        private array $items,
    ) {}

    /**
     * @param int $index
     * @return T
     * @throws OutOfBoundsException
     */
    public function get(int $index): mixed
    {
        return $this->items[$index] ?? throw new OutOfBoundsException("Index {$index} is out of bounds.");
    }
}

/**
 * @template T
 */
final class TypedListVar
{
    public function __construct(
        /** @var list<T> */
        private array $items,
    ) {}

    /**
     * @param int $index
     * @return T
     * @throws OutOfBoundsException
     */
    public function get(int $index): mixed
    {
        return $this->items[$index] ?? throw new OutOfBoundsException("Index {$index} is out of bounds.");
    }
}

function take_string(string $value): void
{
    echo $value;
}

$fromParam = new TypedList(['a', 'b', 'c']);
take_string($fromParam->get(1));

$fromVar = new TypedListVar(['a', 'b', 'c']);
take_string($fromVar->get(1));
