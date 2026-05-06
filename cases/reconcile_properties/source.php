<?php

/**
 * @template Tk
 * @template Tv
 * @api
 */
final class AwaitableIteratorQueue
{
    /**
     * @var list<array{0: Tk, 1: Awaitable<Tv>}>
     */
    public array $items = [];

    /**
     * @var array<string, State<Tv>>
     */
    public array $pending = [];
}

/**
 * @template Tk
 * @template Tv
 * @api
 */
final class AwaitableIterator
{
    /**
     * @var AwaitableIteratorQueue<Tk, Tv>
     */
    public readonly AwaitableIteratorQueue $queue;

    public function __construct()
    {
        $this->queue = new AwaitableIteratorQueue();
    }

    /**
     * @return null|array{0: Tk, 1: Awaitable<Tv>}
     */
    public function consume(): ?array
    {
        if ([] === $this->queue->items) {
            echo 'No items in the queue.';

            return null;
        }

        return $this->queue->items[0];
    }
}

/**
 * @template T
 * @api
 */
final class State
{
    public static string $nextId = 'a';

    public bool $complete = false;

    public bool $handled = false;

    /**
     * @var array<string, (Closure(?Throwable, ?T, string): void)>
     */
    public array $callbacks = [];

    /**
     * @var T|null
     */
    public mixed $result = null;

    public ?Throwable $throwable = null;
}

/**
 * @template T
 * @api
 */
final readonly class Awaitable
{
    public State $state;

    /**
     * @param State<T> $state
     */
    public function __construct(State $state)
    {
        $this->state = $state;
    }
}
