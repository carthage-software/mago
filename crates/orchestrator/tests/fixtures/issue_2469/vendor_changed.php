<?php

namespace Vendor;

final class Item
{
    public int $id = 0;
}

final class OtherItem
{
    public string $id = '';
}

/** @template T */
abstract class Repository
{
    /** @return T|null */
    public function find(int $id): mixed
    {
        return null;
    }
}

/** @extends Repository<Item> */
class ItemRepository extends Repository {}

/** @template T */
interface Contract
{
    /** @return T|null */
    public function first(): mixed;
}

/** @implements Contract<Item> */
class ItemProvider implements Contract
{
    public function first(): mixed
    {
        return null;
    }
}
