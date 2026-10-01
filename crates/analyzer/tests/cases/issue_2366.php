<?php

declare(strict_types=1);

namespace Issue2366;

final readonly class Item
{
    public function __construct(
        public string $id,
        public string $label,
        public bool $flag,
    ) {}

    /** @param array{id: string, label: string, flag: bool} $item */
    public static function fromRaw(array $item): self
    {
        return new self($item['id'], $item['label'], $item['flag']);
    }
}

/**
 * @param list<array{id: string, label: string, flag: bool}> $items
 * @return list<Item>
 */
function mapListInPlace(array $items): array
{
    foreach ($items as $index => $rawItem) {
        $items[$index] = Item::fromRaw($rawItem);
    }

    return $items;
}

/**
 * @param list<array{id: string, label: string, flag: bool}> $items
 * @return array<string, list<Item>>
 */
function mapByKey(array $items): array
{
    if ($items === []) {
        return [];
    }

    $byKey = [];

    foreach ($items as $rawItem) {
        $byKey[$rawItem['id']][] = $rawItem;
    }

    foreach ($byKey as $key => $rawItems) {
        $byKey[$key] = array_map(Item::fromRaw(...), $rawItems);
    }

    return $byKey;
}

/**
 * @param non-empty-list<string> $items
 * @return non-empty-list<int>
 */
function mapNonEmptyList(array $items): array
{
    foreach ($items as $key => $item) {
        $items[$key] = strlen($item);
    }

    return $items;
}

/**
 * @param array<string, list<string>> $items
 * @return array<string, list<int>>
 */
function mapPossiblyEmptyNestedLists(array $items): array
{
    foreach ($items as $key => $list) {
        $items[$key] = array_map(strlen(...), $list);
    }

    return $items;
}

/**
 * @param non-empty-array<string, non-empty-list<string>> $items
 * @return non-empty-array<string, non-empty-list<int>>
 */
function mapNonEmptyNestedLists(array $items): array
{
    foreach ($items as $key => $list) {
        $items[$key] = array_map(strlen(...), $list);
    }

    return $items;
}

/**
 * @param array{first: list<string>, second: list<string>} $items
 * @return array{first: list<int>, second: list<int>}
 */
function mapShape(array $items): array
{
    foreach ($items as $key => $list) {
        $items[$key] = array_map(strlen(...), $list);
    }

    return $items;
}

/**
 * @param non-empty-array<string, list<string>> $items
 * @return non-empty-array<string, list<int>>
 */
function breakLeavesOriginalValues(array $items, bool $stop): array
{
    foreach ($items as $key => $list) {
        if ($stop) {
            break;
        }

        $items[$key] = array_map(strlen(...), $list);
    }

    // @mago-expect analysis:invalid-return-statement
    return $items;
}

/**
 * @param non-empty-array<string, list<string>> $items
 * @return non-empty-array<string, list<int>>
 */
function continueLeavesOriginalValues(array $items, bool $skip): array
{
    foreach ($items as $key => $list) {
        if ($skip) {
            continue;
        }

        $items[$key] = array_map(strlen(...), $list);
    }

    // @mago-expect analysis:invalid-return-statement
    return $items;
}

/**
 * @param non-empty-array<string, list<string>> $items
 * @return non-empty-array<string, list<int>>
 */
function writingAnotherEntryPreservesItsType(array $items): array
{
    foreach ($items as $key => $list) {
        $items[$key] = array_map(strlen(...), $list);
        $items['extra'] = ['original'];
    }

    // @mago-expect analysis:invalid-return-statement
    return $items;
}
