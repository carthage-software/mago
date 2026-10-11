<?php

declare(strict_types=1);

namespace Host;

use Vendor\ItemProvider;
use Vendor\ItemRepository;

function viaExtends(ItemRepository $items): int
{
    return $items->find(1)?->id ?? 0;
}

function viaImplements(ItemProvider $items): int
{
    return $items->first()?->id ?? 0;
}
