<?php

declare(strict_types=1);

class Item
{
}

function i_take_items(Item ...$_items): void
{
}

/**
 * @param array<Item|null|''|false|0> $items
 */
function process(array $items): void
{
    i_take_items(...array_filter(
        $items,
        /**
         * @assert-if-true Item $item
         */
        fn($item) => $item instanceof Item,
    ));
}
