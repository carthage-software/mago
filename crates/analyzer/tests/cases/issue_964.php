<?php

declare(strict_types=1);

/**
 * @template TKey of array-key
 * @template TValue
 */
class Foo
{
    /** @var iterable<TKey,TValue> */
    private iterable $items;

    /**
     * @param iterable<TKey, TValue> $items
     */
    public function __construct(iterable $items = [])
    {
        $this->items = $items;
    }

    public function lateBinding(): static
    {
        return new static($this->items); // @mago-expect analysis:unsafe-instantiation
    }
}

final class Test
{
    public function run(): void
    {
        /** @var Foo<int, string> $data */
        $data = new Foo([]);

        // lateBinding() should return Foo<int, string>, not Foo<array-key, mixed>
        $result = $data->lateBinding();

        $this->acceptFoo($result);
    }

    /**
     * @param Foo<int, string> $_foo
     */
    private function acceptFoo(Foo $_foo): void
    {
    }
}
