<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenTree
{
    /**
     * @param T $value
     * @param list<GenTree<T>> $children
     */
    public function __construct(public mixed $value, public array $children = [])
    {
    }

    /** @return T */
    public function getValue(): mixed
    {
        return $this->value;
    }
}

function takes_int_tree(int $n): void
{
}

$t = new GenTree(1, [new GenTree(2), new GenTree(3, [new GenTree(4)])]);
takes_int_tree($t->getValue());
