<?php

declare(strict_types=1);

namespace Issue2440;

/** @template T */
interface Paginator
{
    /** @return list<T> */
    public function items(): array;
}

interface Adapter
{
    public function count(): int;
}

/** @param Paginator<int>|Adapter $source */
function count_items(Paginator|Adapter $source): int
{
    if ($source instanceof Adapter) {
        return $source->count();
    }

    return count($source->items());
}

/** @param (Paginator<int>&Adapter)|Adapter $source */
function count_intersection(Paginator|Adapter $source): int
{
    return $source->count();
}

/**
 * @param (Paginator<int>&Adapter)|(Paginator<string>&Adapter) $source
 * @return list<int|string>
 */
function intersection_items(Paginator $source): array
{
    if ($source->count() > 0) {
        return $source->items();
    }

    return [];
}

/**
 * @param Paginator<int>|Adapter $source
 * @return list<int>
 */
function paginator_items(Paginator|Adapter $source): array
{
    if ($source instanceof Paginator) {
        return $source->items();
    }

    return [];
}

/**
 * @param Paginator<int>|Adapter $source
 * @return list<int>
 */
function intersection_after_check(Paginator|Adapter $source): array
{
    if ($source instanceof Adapter && $source instanceof Paginator) {
        $source->count();

        return $source->items();
    }

    return [];
}

/** @template T */
abstract class Collection
{
    /** @return list<T> */
    abstract public function items(): array;
}

/** @param Collection<int>|Adapter $source */
function count_collection(Collection|Adapter $source): int
{
    if ($source instanceof Adapter) {
        return $source->count();
    }

    return count($source->items());
}
