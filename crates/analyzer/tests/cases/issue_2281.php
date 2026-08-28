<?php

declare(strict_types=1);

final class Item
{
}

/** @param list<Item> $items */
function takeList(array $items): void
{
}

/** @param list<list<Item>> $groups */
function annotatedSpread(array $groups): void
{
    takeList(\array_merge(...$groups));
}

/** @param list<list<Item>> $groups */
function loopThenSpread(array $groups): void
{
    $collected = [];

    foreach ($groups as $group) {
        $collected[] = $group;
    }

    takeList(\array_merge(...$collected));
}
