<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenStore
{
    /** @var list<T> */
    public array $items = [];

    /** @param T $item */
    public function add(mixed $item): void
    {
        $this->items[] = $item;
    }

    /** @return list<T> */
    public function all(): array
    {
        return $this->items;
    }
}

/** @var GenStore<int> $st */
$st = new GenStore();
$st->add(1);
$st->add(2);
foreach ($st->all() as $n) {
    echo $n + 1;
}
