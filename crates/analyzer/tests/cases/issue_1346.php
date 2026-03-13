<?php

declare(strict_types=1);

interface HandlerInterface1346 {}

final class Pipeline1346
{
    /**
     * @var WeakMap<HandlerInterface1346, HandlerInterface1346>
     */
    private WeakMap $cache;

    public function __construct()
    {
        // new WeakMap() produces WeakMap<object, mixed> by default,
        // but the property type narrows it — no coercion warning.
        $this->cache = new WeakMap();
    }

    public function get(HandlerInterface1346 $h): HandlerInterface1346
    {
        return $this->cache[$h] ?? $h;
    }

    public function set(HandlerInterface1346 $h, HandlerInterface1346 $c): void
    {
        $this->cache[$h] = $c;
    }
}

/**
 * @template T
 */
class Stack1346
{
    /** @var list<T> */
    private array $items = [];

    public function __construct() {}

    /**
     * @param T $item
     */
    public function push(mixed $item): void
    {
        $this->items[] = $item;
    }

    /**
     * @return T
     */
    public function pop(): mixed
    {
        $item = array_pop($this->items);
        if ($item === null) {
            /** @mago-expect analysis:unhandled-thrown-type */
            throw new \RuntimeException('Stack is empty');
        }
        return $item;
    }
}

/**
 * @template TNode
 *
 * @param TNode $start
 *
 * @return list<TNode>
 */
function dfs1346(mixed $start): array
{
    $result = [];
    /** @var Stack1346<TNode> $stack */
    $stack = new Stack1346();
    $stack->push($start);

    while (true) {
        $node = $stack->pop();
        $result[] = $node;
        break;
    }

    return $result;
}

// Using generic container without type annotation — should still work
// with constraint defaults (push accepts mixed).
$untyped_stack = new Stack1346();
$untyped_stack->push('hello');
$untyped_stack->push('world');
