<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenStoreWrong
{
    /** @var list<T> */
    public array $items = [];

    /** @param T $item */
    public function add(mixed $item): void
    {
        $this->items[] = $item;
    }
}

/**
 * @param GenStoreWrong<int> $st
 */
function inserter(GenStoreWrong $st): void
{
    $st->add('not int');
}
