<?php

declare(strict_types=1);

final class Item
{
    public string $prop = '';
    public ?string $nullable = null;
}

final class Container
{
    public function __construct(public Item $item = new Item()) {}
}

/** @param list<Item> $list */
function nullable_result(array $list): ?string
{
    return $list[0]->prop ?? null;
}

/** @param list<Item> $list */
function repeated_result(array $list): void
{
    // @mago-expect analysis:possibly-invalid-argument
    echo strlen($list[0]->prop ?? 42);
    // @mago-expect analysis:possibly-invalid-argument
    echo strlen($list[0]->prop ?? 42);
}

/** @param array<string, Item> $items */
function keyed_result(array $items): ?string
{
    return $items['missing']->prop ?? null;
}

/** @param array{item?: Item} $items */
function optional_result(array $items): ?string
{
    return $items['item']->prop ?? null;
}

/** @param list<Container> $list */
function chained_result(array $list): ?string
{
    return $list[0]->item->prop ?? null;
}

/** @param non-empty-list<Item> $list */
function present_list_result(array $list): string
{
    // @mago-expect analysis:redundant-null-coalesce
    return $list[0]->prop ?? '';
}

/** @param array{item: Item} $items */
function present_shape_result(array $items): string
{
    // @mago-expect analysis:redundant-null-coalesce
    return $items['item']->prop ?? '';
}

/** @param non-empty-list<Item> $list */
function nullable_property_result(array $list): string
{
    return $list[0]->nullable ?? '';
}
