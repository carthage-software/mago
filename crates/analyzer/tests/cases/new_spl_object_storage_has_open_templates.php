<?php

declare(strict_types=1);

final class Registry
{
    /** @var SplObjectStorage<object, int> */
    private SplObjectStorage $items;

    public function __construct()
    {
        $this->items = new SplObjectStorage();
    }

    public function add(object $item): void
    {
        $this->items[$item] = 1;
    }
}

function distinct(object $a, object $b): int
{
    $seen = new SplObjectStorage();
    $seen[$a] = true;
    $seen[$b] = true;

    return count($seen);
}
