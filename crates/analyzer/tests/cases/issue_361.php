<?php

class Item {}

class Example1
{
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
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
    // @mago-expect analysis:write-only-property
    /** @var array<Item> */
    private readonly array $defaultLineItems;

    /** @param Generator<Item> $defaultLineItems */
    public function __construct(Generator $defaultLineItems)
    {
        $this->defaultLineItems = [...$defaultLineItems];
    }
}
