<?php

declare(strict_types=1);

namespace Issue2305;

final class Item
{
    public function __construct(
        public string $key,
        public int $value,
    ) {}
}

final class Collection
{
    /** @param non-empty-list<Item> $items */
    public function __construct(public array $items) {}
}

final class Repro
{
    /** @param non-empty-list<Item> $source */
    public function build(array $source): Collection
    {
        $byKey = [];
        foreach ($source as $item) {
            $byKey[$item->key] = $item;
        }

        return new Collection(array_values($byKey));
    }

    /** @param list<Item> $source */
    public function possiblyEmptySource(array $source): Collection
    {
        $byKey = [];
        foreach ($source as $item) {
            $byKey[$item->key] = $item;
        }

        // @mago-expect analysis:possibly-invalid-argument
        return new Collection(array_values($byKey));
    }

    /** @param non-empty-list<Item> $source */
    public function conditionalWrite(array $source): Collection
    {
        $byKey = [];
        foreach ($source as $item) {
            if ($item->value > 0) {
                $byKey[$item->key] = $item;
            }
        }

        // @mago-expect analysis:possibly-invalid-argument
        return new Collection(array_values($byKey));
    }

    /** @param non-empty-list<Item> $source */
    public function resetAfterWrite(array $source): Collection
    {
        $byKey = [];
        foreach ($source as $item) {
            $byKey[$item->key] = $item;
            $byKey = [];
        }

        // @mago-expect analysis:possibly-invalid-argument
        return new Collection(array_values($byKey));
    }

    /** @param non-empty-array<string, Item> $items */
    public function contains(array $items, string $key): bool
    {
        return isset($items[$key]);
    }
}
