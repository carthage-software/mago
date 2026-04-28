<?php

declare(strict_types=1);

final class Item
{
    public function name(): string
    {
        return 'item';
    }
}

function flow_nullsafe_on_nonnull(Item $i): null|string
{
    return $i?->name();
}
