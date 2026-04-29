<?php

declare(strict_types=1);

/**
 * @template T of int|string
 */
final class Stack
{
    /** @var list<T> */
    public array $items = [];
}

final class Holder
{
    /** @var Stack<int> */
    public Stack $stack;

    public function __construct()
    {
        $this->stack = new Stack();
    }
}

$h = new Holder();
$h->stack->items[] = 1;
echo count($h->stack->items);
