<?php

declare(strict_types=1);

final class Item
{
    public string $prop = '';
}

/** @param list<Item> $list */
function nullsafe_result(array $list): string
{
    $value = $list[0]?->prop;

    return $value === null ? 'missing' : $value;
}
