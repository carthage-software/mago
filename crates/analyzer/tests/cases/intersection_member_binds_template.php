<?php

declare(strict_types=1);

interface Marker
{
}

/**
 * @template T of object
 */
interface Collection
{
    /**
     * @return T
     */
    public function get(): object;
}

/**
 * @template T of object
 *
 * @param Collection<T> $collection
 *
 * @return T
 */
function first(Collection $collection): object
{
    return $collection->get();
}

/**
 * @param Marker&Collection $marked
 * @param Collection $plain
 */
function both($marked, $plain): object
{
    first($plain);

    return first($marked);
}

final class Item
{
}

/**
 * @param Marker&Collection<Item> $marked
 */
function typed($marked): Item
{
    return first($marked);
}
