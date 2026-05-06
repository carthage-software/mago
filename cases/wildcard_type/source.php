<?php

declare(strict_types=1);

/**
 * @template T
 */
class Collection
{
    /** @var list<T> */
    private array $items = [];

    /** @param T $item */
    public function add(mixed $item): void
    {
        $this->items[] = $item;
    }

    /** @return list<T> */
    public function all(): array
    {
        return $this->items;
    }
}

/**
 * @param Collection<*> $collection
 * @return list<mixed>
 */
function get_all_items_asterisk(Collection $collection): array
{
    return $collection->all();
}

/**
 * @param Collection<_> $collection
 * @return list<mixed>
 */
function get_all_items_underscore(Collection $collection): array
{
    return $collection->all();
}

/**
 * @param array<string, *> $data
 * @return mixed
 */
function get_value(array $data, string $key): mixed
{
    return $data[$key];
}

/** @var * $anything */
$anything = null;

/** @var _ $also_anything */
$also_anything = null;
