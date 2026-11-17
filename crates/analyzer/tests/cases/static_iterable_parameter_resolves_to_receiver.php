<?php

declare(strict_types=1);

/**
 * @implements IteratorAggregate<int, static>
 */
abstract class StaticIterableCollection implements IteratorAggregate
{
    public function getIterator(): Iterator
    {
        return new ArrayIterator([$this]);
    }
}

final class StaticIterableLeaf extends StaticIterableCollection
{
    public function value(): int
    {
        return 1;
    }
}

function iterate_static_leaf(StaticIterableLeaf $collection): int
{
    foreach ($collection as $item) {
        return $item->value();
    }

    return 0;
}

/**
 * @return array<int, StaticIterableLeaf>
 */
function collect_static_leaf(StaticIterableLeaf $collection): array
{
    return iterator_to_array($collection);
}
