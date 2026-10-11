<?php

namespace Vendor;

/** @extends Repository<Item> */
class ItemRepository extends Repository {}

/** @extends Repository<Item> */
class BoundRepository extends Repository {}

/**
 * @template T
 * @extends Repository<T>
 */
class GenericRepository extends Repository {}

/** @implements \IteratorAggregate<int, Item> */
class ItemList implements \IteratorAggregate {}

/** @extends \ArrayObject<int, Item> */
class ItemArray extends \ArrayObject {}

/** @implements FirstContract<Item> */
class MultipleBindings implements FirstContract, SecondContract {}

/** @extends FirstContract<Item> */
interface ItemContract extends FirstContract {}
