<?php

final class A
{
}

final class B
{
    /**
     * @return list<array{k: Index, v: Item}>
     */
    public function getItems(): array
    {
        return [];
    }
}

final class C
{
}

enum Index
{
    case Foo;
    case Bar;
    case Baz;
}

class Item
{
}

class Example
{
    /** @return WeakMap<Index, Item> */
    private function doThing(A|B|C $dto): WeakMap
    {
        return match ($dto::class) {
            A::class => (static function (A $_): WeakMap {
                /** @var WeakMap<Index, Item> */
                return new WeakMap();
            })($dto),
            B::class => (static function (B $dto): WeakMap {
                /** @var WeakMap<Index, Item> $result */
                $result = new WeakMap();

                foreach ($dto->getItems() as $item) {
                    $result->offsetSet($item['k'], $item['v']);
                }

                return $result;
            })($dto),
            C::class => (static function (C $_): WeakMap {
                /** @var WeakMap<Index, Item> $result */
                $result = new WeakMap();

                return $result;
            })($dto),
        };
    }
}
