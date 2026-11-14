<?php

declare(strict_types=1);

final class Item extends stdClass
{
    public int $count = 0;
}

function write(stdClass $plain, Item $item): void
{
    $plain->extra = 'x';
    $item->extra = 'x';
}
