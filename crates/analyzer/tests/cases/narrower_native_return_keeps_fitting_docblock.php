<?php

declare(strict_types=1);

abstract class A
{
    /** @return list<int>|false */
    abstract public function items(): array|false;

    /** @return int|non-empty-string */
    abstract public function id(): int|string;
}

final class B extends A
{
    #[Override]
    public function items(): array
    {
        return [1];
    }

    #[Override]
    public function id(): string
    {
        return 'x';
    }
}

/** @param list<int> $items */
function take(array $items, string $id): string
{
    return $id . count($items);
}

/** @return non-empty-string */
function test(B $b): string
{
    take($b->items(), $b->id());

    return $b->id();
}
