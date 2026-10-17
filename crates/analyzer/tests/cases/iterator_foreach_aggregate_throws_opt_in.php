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

final class AggregateItemList implements \IteratorAggregate // @mago-expect analysis:missing-template-parameter
{
    public function getIterator(): ItemCollection
    {
        return new ItemCollection();
    }
}

/** @throws \Exception */
function processAggregateItems(AggregateItemList $list): int
{
    $total = 0;
    foreach ($list as $item) {
        $total += $item->getId();
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

/** @throws \Exception */
function processCyclic(CyclicAggregate $cyclic): void
{
    foreach ($cyclic as $item) { // @mago-expect analysis:mixed-assignment
    }
}
