<?php

/**
 * @template TKey of array-key
 * @template TValue
 *
 * @implements ArrayAccess<TKey, TValue>
 * @implements IteratorAggregate<TKey, TValue>
 */
class ArrayCollection implements ArrayAccess, Countable, IteratorAggregate
{
    /** @var array<TKey, TValue> */
    private array $elements = [];

    public function __construct()
    {
        $this->elements = [];
    }

    /**
     * @template K of array-key
     * @template V
     *
     * @param array<K, V> $elements
     *
     * @return self<K, V>
     */
    public static function fromArray(array $elements): self
    {
        $collection = new self();
        foreach ($elements as $key => $value) {
            $collection[$key] = $value;
        }
        return $collection;
    }

    /** @param TKey $offset */
    public function offsetExists($offset): bool
    {
        return isset($this->elements[$offset]);
    }

    /** @param TKey $offset */
    public function offsetGet($offset): mixed
    {
        return $this->elements[$offset] ?? null;
    }

    /**
     * @param TKey|null $offset
     * @param TValue $value
     *
     * @mago-expect analysis:possibly-null-array-index
     * @mago-expect analysis:invalid-property-assignment-value
     */
    public function offsetSet($offset, $value): void
    {
        $this->elements[$offset] = $value;
    }

    /** @param TKey $offset */
    public function offsetUnset($offset): void
    {
        unset($this->elements[$offset]);
    }

    /** @return int<0, max> */
    public function count(): int
    {
        return count($this->elements);
    }

    /** @return Traversable<TKey, TValue> */
    public function getIterator(): Traversable
    {
        return new ArrayIterator($this->elements);
    }
}

/**
 * @param ArrayCollection<non-negative-int, string> $collection
 */
function appendToCollection(ArrayCollection $collection, string $value): void
{
    $collection[] = $value;
}

/** @var ArrayCollection<non-negative-int, string> $collection */
$collection = ArrayCollection::fromArray([
    0 => 'a',
    1 => 'b',
    2 => 'c',
]);

appendToCollection($collection, 'd');
