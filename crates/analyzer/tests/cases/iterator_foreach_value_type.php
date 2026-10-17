<?php

declare(strict_types=1);

namespace App;

final class Item
{
    public function getId(): int
    {
        return 42;
    }
}

/**
 * Legacy collection implementing \Iterator without generic parameters.
 * key() returns mixed, current() returns a concrete Item.
 */
final class ItemCollection implements \Iterator // @mago-expect analysis:missing-template-parameter
{
    /** @var list<Item> */
    private array $items = [];
    private int $position = 0;

    public function current(): Item
    {
        return $this->items[$this->position];
    }

    public function next(): void
    {
        $this->position++;
    }

    public function key(): mixed
    {
        return $this->position;
    }

    public function valid(): bool
    {
        return isset($this->items[$this->position]);
    }

    public function rewind(): void
    {
        $this->position = 0;
    }
}

function processItems(ItemCollection $collection): int
{
    $total = 0;
    foreach ($collection as $item) {
        $total += $item->getId();
        $item->unknownMethod(); // @mago-expect analysis:non-existent-method
    }

    return $total;
}

class Entity
{
    public function getId(): int
    {
        return 1;
    }
}

final class Order extends Entity
{
}

/**
 * Generic collection implementing \Iterator without @implements arguments.
 *
 * @template TEntity of Entity
 */
abstract class EntityList implements \Iterator // @mago-expect analysis:missing-template-parameter
{
    /** @var list<TEntity> */
    protected array $entities = [];
    private int $position = 0;

    /** @return TEntity */
    public function current(): Entity
    {
        return $this->entities[$this->position];
    }

    public function next(): void
    {
        $this->position++;
    }

    public function key(): int
    {
        return $this->position;
    }

    public function valid(): bool
    {
        return isset($this->entities[$this->position]);
    }

    public function rewind(): void
    {
        $this->position = 0;
    }
}

/**
 * Child binding the template parameter.
 *
 * @extends EntityList<Order>
 */
final class OrderList extends EntityList
{
}

/**
 * Child without generic arguments.
 */
final class LegacyOrderList extends EntityList // @mago-expect analysis:missing-template-parameter
{
}

function processBoundOrders(OrderList $list): int
{
    $sum = 0;
    foreach ($list as $order) {
        $sum += $order->getId();
    }

    return $sum;
}

function processUnboundOrders(LegacyOrderList $list): int
{
    $sum = 0;
    foreach ($list as $order) {
        $sum += $order->getId();
    }

    return $sum;
}

final class AggregateItemList implements \IteratorAggregate // @mago-expect analysis:missing-template-parameter
{
    public function getIterator(): ItemCollection
    {
        return new ItemCollection();
    }
}

function processAggregateItems(AggregateItemList $list): int
{
    $total = 0;
    foreach ($list as $item) {
        $total += $item->getId();
        $item->unknownMethod(); // @mago-expect analysis:non-existent-method
    }

    return $total;
}

final class CyclicAggregate implements \IteratorAggregate // @mago-expect analysis:missing-template-parameter
{
    public function getIterator(): self
    {
        return $this;
    }
}

function processCyclic(CyclicAggregate $cyclic): void
{
    foreach ($cyclic as $item) { // @mago-expect analysis:mixed-assignment
        // Must not crash or overflow stack.
    }
}

