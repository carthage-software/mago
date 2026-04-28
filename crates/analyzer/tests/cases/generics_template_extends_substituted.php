<?php

declare(strict_types=1);

/**
 * @template K
 * @template V
 */
abstract class GenMapBase
{
    /** @return list<K> */
    abstract public function keys(): array;

    /** @return list<V> */
    abstract public function values(): array;
}

/**
 * @template V
 *
 * @extends GenMapBase<string, V>
 */
abstract class GenStrKeyMap extends GenMapBase
{
}

/**
 * @extends GenStrKeyMap<int>
 */
final class GenStrIntMap extends GenStrKeyMap
{
    public function keys(): array
    {
        return ['a', 'b'];
    }

    public function values(): array
    {
        return [1, 2];
    }
}

$m = new GenStrIntMap();
foreach ($m->keys() as $k) {
    echo $k;
}
foreach ($m->values() as $v) {
    echo $v;
}
