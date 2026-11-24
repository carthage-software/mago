<?php

declare(strict_types=1);

/** @template T */
abstract class ComposedBase
{
    /** @var T */
    public mixed $value;

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

/**
 * @template U
 * @extends ComposedBase<list<U>>
 */
abstract class ComposedList extends ComposedBase {}

/**
 * @template K
 * @template V
 * @extends ComposedBase<array<K, V>>
 */
abstract class ComposedMap extends ComposedBase {}

/**
 * @template TModel of object
 * @extends ComposedList<TModel>
 */
abstract class ComposedLeaf extends ComposedList {}

/** @param list<DateTime> $_ */
function composed_takes_dates(array $_): void {}

/** @param array<int, string> $_ */
function composed_takes_map(array $_): void {}

/**
 * @param ComposedList<DateTime> $list
 * @param ComposedMap<int, string> $map
 * @param ComposedLeaf<DateTime> $leaf
 */
function composed_read(ComposedList $list, ComposedMap $map, ComposedLeaf $leaf): void
{
    composed_takes_dates($list->value);
    composed_takes_map($map->value);
    composed_takes_dates($leaf->value);

    $list->value = [1];
    $leaf->value = [new stdClass()];
}
