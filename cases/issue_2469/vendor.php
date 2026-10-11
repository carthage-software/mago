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

class ItemRepository extends Repository {}

/** @extends Repository<OtherItem> */
class BoundRepository extends Repository {}

/** @template T */
class GenericRepository extends Repository {}

class ItemList implements \IteratorAggregate
{
    public function getIterator(): \Iterator
    {
        return new \ArrayIterator([]);
    }
}

class ItemArray extends \ArrayObject {}

/** @template T */
interface FirstContract
{
    /** @return T */
    public function first(): mixed;
}

/** @template T */
interface SecondContract
{
    /** @return T */
    public function second(): mixed;
}

/**
 * @implements FirstContract<OtherItem>
 * @implements SecondContract<string>
 */
class MultipleBindings implements FirstContract, SecondContract
{
    public function first(): mixed
    {
        return new OtherItem();
    }

    public function second(): mixed
    {
        return '';
    }
}

interface ItemContract extends FirstContract {}
