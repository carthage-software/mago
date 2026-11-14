<?php

declare(strict_types=1);

/**
 * @return list<string>
 *
 * @no-named-arguments
 */
function func(string $item1, string $item2, string ...$otherItems): array
{
    return [$item1, $item2, ...$otherItems];
}

/**
 * @param non-empty-list<string> $existingItems
 * @param list<string> $maybeEmpty
 */
function call(array $existingItems, array $maybeEmpty, string $newItem): void
{
    func(...[...$existingItems, $newItem]);
    func(...[$newItem, ...$existingItems]);
    func(...[...$maybeEmpty, $newItem, $newItem]);
}
