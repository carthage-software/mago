<?php

declare(strict_types=1);

namespace Host;

use Vendor\BoundRepository;
use Vendor\GenericRepository;
use Vendor\Item;
use Vendor\ItemArray;
use Vendor\ItemContract;
use Vendor\ItemList;
use Vendor\ItemRepository;
use Vendor\MultipleBindings;

final class DescendantRepository extends BoundRepository {}

function viaExtends(ItemRepository $items): int
{
    return $items->find(1)?->id ?? 0;
}

function viaExistingBinding(BoundRepository $items): int
{
    return $items->find(1)?->id ?? 0;
}

function viaDescendant(DescendantRepository $items): int
{
    return $items->find(1)?->id ?? 0;
}

/** @param GenericRepository<Item> $items */
function viaTemplateForwarding(GenericRepository $items): int
{
    return $items->find(1)?->id ?? 0;
}

function viaImplements(ItemList $items): int
{
    foreach ($items as $item) {
        return $item->id;
    }

    return 0;
}

function viaArrayObject(ItemArray $items): int
{
    $items[0] = new Item();

    return $items[0]->id;
}

function rejectsInvalidArrayObjectValue(ItemArray $items): void
{
    // @mago-expect analysis:invalid-array-access-assignment-value
    $items[0] = 5;
}

function viaPatchedInterface(MultipleBindings $items): int
{
    return $items->first()->id;
}

function viaUntouchedInterface(MultipleBindings $items): string
{
    return $items->second();
}

function viaInterfaceExtends(ItemContract $items): int
{
    return $items->first()->id;
}
