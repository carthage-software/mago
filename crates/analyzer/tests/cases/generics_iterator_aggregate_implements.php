<?php

declare(strict_types=1);

/**
 * @template V
 *
 * @implements IteratorAggregate<int, V>
 */
final class GenListAggr implements IteratorAggregate
{
    /** @param list<V> $items */
    public function __construct(private array $items)
    {
    }

    public function getIterator(): Iterator
    {
        return new ArrayIterator($this->items);
    }
}

$g = new GenListAggr(['a', 'b']);
foreach ($g as $v) {
    echo $v;
}
