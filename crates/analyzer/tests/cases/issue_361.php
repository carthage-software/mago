<?php

class Item
{
}

class Example1
{
    /** @var list<Item> */
    private readonly array $defaultLineItems;

    /** @param iterable<int, Item> $defaultLineItems */
    public function __construct(iterable $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

class Example1Array
{
    /** @var list<Item> */
    private readonly array $defaultLineItems;

    /** @param array<int, Item> $defaultLineItems */
    public function __construct(iterable $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

class Example2
{
    /** @var array<string, Item> */
    private readonly array $defaultLineItems;

    /** @param iterable<string, Item> $defaultLineItems */
    public function __construct(iterable $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

class Example2Array
{
    /** @var array<string, Item> */
    private readonly array $defaultLineItems;

    /** @param iterable<string, Item> $defaultLineItems */
    public function __construct(iterable $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

class Example3
{
    /** @var array<Item> */
    private readonly array $defaultLineItems;

    /** @param iterable<array-key, Item> $defaultLineItems */
    public function __construct(iterable $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

class Example3Array
{
    /** @var array<Item> */
    private readonly array $defaultLineItems;

    /** @param iterable<array-key, Item> $defaultLineItems */
    public function __construct(iterable $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

/**
 * @mago-expect analysis:invalid-array-element-key
 */
class Example4
{
    /** @var array<Item> */
    private readonly array $defaultLineItems;

    /** @param iterable<Item> $defaultLineItems */
    public function __construct(iterable $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

/**
 * @mago-expect analysis:invalid-array-element-key
 */
class Example4Traversable
{
    /** @var array<Item> */
    private readonly array $defaultLineItems;

    /** @param Traversable<Item> $defaultLineItems */
    public function __construct(Traversable $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

/**
 * @mago-expect analysis:invalid-array-element-key
 */
class Example4IteratorAggregate
{
    /** @var array<Item> */
    private readonly array $defaultLineItems;

    /** @param IteratorAggregate<Item> $defaultLineItems */
    public function __construct(IteratorAggregate $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}

/**
 * @mago-expect analysis:invalid-array-element-key
 */
class Example4Generator
{
    /** @var array<Item> */
    private readonly array $defaultLineItems;

    /** @param Generator<Item> $defaultLineItems */
    public function __construct(Generator $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}
