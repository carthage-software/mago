<?php

declare(strict_types=1);

/**
 * @template K of array-key
 * @template V
 *
 * @implements IteratorAggregate<K, V>
 */
final class GenIterAggr implements IteratorAggregate
{
    /** @param array<K, V> $items */
    public function __construct(private array $items)
    {
    }

    /** @return Iterator<K, V> */
    public function getIterator(): Iterator
    {
        return new ArrayIterator($this->items);
    }
}

$gi = new GenIterAggr(['a' => 1, 'b' => 2]);
foreach ($gi as $k => $v) {
    echo $k . ' => ' . $v;
}
