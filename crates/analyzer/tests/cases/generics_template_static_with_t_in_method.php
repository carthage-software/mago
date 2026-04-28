<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenStaticT
{
    /**
     * @param list<T> $items
     */
    public function __construct(public array $items)
    {
    }

    /**
     * @template U
     *
     * @param list<U> $items
     *
     * @return GenStaticT<U>
     */
    public static function fromList(array $items): GenStaticT
    {
        return new GenStaticT($items);
    }
}

/** @var GenStaticT<int> $g */
$g = GenStaticT::fromList([1, 2, 3]);
foreach ($g->items as $n) {
    echo $n + 1;
}
