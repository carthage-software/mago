<?php

declare(strict_types=1);

final class Holder
{
    /** @var list<int> */
    public array $items = [];
}

/**
 * @return non-empty-list<int>|null
 */
function truthy(?Holder $o): ?array
{
    if (!$o?->items) {
        return null;
    }

    return $o->items;
}

/**
 * @return non-empty-list<int>|null
 */
function not_empty(?Holder $o): ?array
{
    if ($o?->items === [] || $o === null) {
        return null;
    }

    return $o->items;
}
