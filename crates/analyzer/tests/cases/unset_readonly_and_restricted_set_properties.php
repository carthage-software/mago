<?php

declare(strict_types=1);

final class Frozen
{
    /** @param array<string, int> $items */
    public function __construct(
        public readonly int $foo = 0,
        public readonly array $items = [],
    ) {}
}

final class Restricted
{
    public private(set) ?int $value = null;

    public function reset(): void
    {
        unset($this->value);
    }
}

function read_then_unset(Frozen $frozen): int
{
    $old = $frozen->foo;
    /** @mago-expect analysis:invalid-unset */
    unset($frozen->foo);

    return $old;
}

function unset_item(Frozen $frozen, string $key): void
{
    /** @mago-expect analysis:invalid-unset */
    unset($frozen->items[$key]);
}

function unset_restricted(Restricted $restricted): void
{
    /** @mago-expect analysis:invalid-unset */
    unset($restricted->value);
}

final class Pool
{
    /** @var WeakMap<object, int> */
    private readonly WeakMap $checkedOut;

    public function __construct()
    {
        $this->checkedOut = new WeakMap();
    }

    public function release(object $item): void
    {
        unset($this->checkedOut[$item]);
    }
}
