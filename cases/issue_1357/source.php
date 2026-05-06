<?php

namespace Psl\Dict {
    /**
     * @template Tk of array-key
     * @template Tv
     *
     * @param iterable<Tk, Tv> $iterable
     * @param iterable<Tk> $keys
     *
     * @return array<Tk, Tv>
     */
    function select_keys(iterable $iterable, iterable $keys): array
    {
        return select_keys($iterable, $keys);
    }
}

namespace App {
    use function Psl\Dict\select_keys;

    /**
     * @return array{id: string, name: string}
     */
    function getSelected(): array
    {
        $source = [
            'id' => 'abc',
            'name' => 'foo',
            'extra' => 'bar',
        ];

        return select_keys($source, ['id', 'name']);
    }

    /**
     * @template K as array-key
     * @template V
     *
     * @param array<K, int|string>|\Iterator<K, V> $source
     *
     * @return array{id?: int|string|V, name?: int|string|V}
     */
    function getSelectedGeneric(iterable $source): array
    {
        return select_keys($source, ['id', 'name']);
    }
}
