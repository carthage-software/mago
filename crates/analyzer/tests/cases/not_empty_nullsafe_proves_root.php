<?php

declare(strict_types=1);

final class Item
{
    public function count(): int
    {
        return 1;
    }
}

function total(?Item $item): int
{
    if (!empty($item?->count())) {
        return $item->count();
    }

    return 0;
}
