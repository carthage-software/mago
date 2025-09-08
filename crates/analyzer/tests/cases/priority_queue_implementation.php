<?php

/**
 * Returns the largest element of the given list, or null if the
 * list is empty.
 *
 * @template T of int|float
 *
 * @param list<T> $numbers
 *
 * @return ($numbers is non-empty-list<T> ? T : null)
 *
 * @pure
 */
function max_value(array $numbers): null|int|float
{
    return max_value($numbers);
}

/**
 * An interface representing a queue data structure ( FIFO ).
 *
 * @template T
 */
interface QueueInterface extends Countable
{
    /**
     * Adds a node to the queue.
     *
     * @param T $node
     */
    public function enqueue(mixed $node): void;

    /**
     * Retrieves, but does not remove, the node at the head of this queue,
     * or returns null if this queue is empty.
     *
     * @return null|T
     */
    public function peek(): mixed;

    /**
     * Retrieves and removes the node at the head of this queue,
     * or returns null if this queue is empty.
     *
     * @return null|T
     */
    public function pull(): mixed;

    /**
     * Retrieves and removes the node at the head of this queue.
     *
     * @return T
     */
    public function dequeue(): mixed;

    /**
     * Count the nodes in the queue.
     *
     * @return int<0, max>
     */
    #[Override]
    public function count(): int;
}

/**
 * @template T
 *
 * @extends QueueInterface<T>
 */
interface PriorityQueueInterface extends QueueInterface
{
    /**
     * Adds a node to the queue.
     *
     * @param T $node
     */
    #[Override]
    public function enqueue(mixed $node, int $priority = 0): void;
}

/**
 * @template T
 *
 * @implements PriorityQueueInterface<T>
 */
final class PriorityQueue implements PriorityQueueInterface
{
    /**
     * @var array<int, non-empty-list<T>>
     */
    private array $queue = [];

    /**
     * Adds a node to the queue.
     *
     * @param T $node
     *
     * @psalm-external-mutation-free
     */
    #[Override]
    public function enqueue(mixed $node, int $priority = 0): void
    {
        $nodes = $this->queue[$priority] ?? [];
        $nodes[] = $node;

        $this->queue[$priority] = $nodes;
    }

    /**
     * Retrieves, but does not remove, the node at the head of this queue,
     * or returns null if this queue is empty.
     *
     * @return null|T
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function peek(): mixed
    {
        if (0 === $this->count()) {
            return null;
        }

        $keys = array_keys($this->queue);

        // Retrieve the highest priority.
        $priority = max_value($keys) ?? 0;

        // Retrieve the list of nodes with the priority `$priority`.
        $nodes = $this->queue[$priority] ?? [];

        // Retrieve the first node of the list.
        return $nodes[0] ?? null;
    }

    /**
     * Retrieves and removes the node at the head of this queue,
     * or returns null if this queue is empty.
     *
     * @return null|T
     *
     * @psalm-external-mutation-free
     */
    #[Override]
    public function pull(): mixed
    {
        try {
            return $this->dequeue();
        } catch (UnderflowException) {
            return null;
        }
    }

    /**
     * Dequeues a node from the queue.
     *
     * @throws UnderflowException If the queue is empty.
     *
     * @return T
     *
     * @psalm-external-mutation-free
     */
    #[Override]
    public function dequeue(): mixed
    {
        if (0 === $this->count()) {
            throw new UnderflowException('Cannot dequeue a node from an empty queue.');
        }

        /**
         * retrieve the highest priority.
         *
         * @var int
         */
        $priority = max_value(array_keys($this->queue));

        /**
         * retrieve the list of nodes with the priority `$priority`.
         */
        $nodes = $this->queue[$priority];

        /**
         * shift the first node out.
         */
        $node = array_shift($nodes);

        /**
         * If the list contained only this node, remove the list of nodes with priority `$priority`.
         */
        if ([] === $nodes) {
            unset($this->queue[$priority]);

            return $node;
        }

        $this->queue[$priority] = $nodes;

        return $node;
    }

    /**
     * Count the nodes in the queue.
     *
     * @return int<0, max>
     *
     * @psalm-mutation-free
     */
    #[Override]
    public function count(): int
    {
        $count = 0;
        foreach ($this->queue as $list) {
            $count += count($list);
        }

        /** @var int<0, max> */
        return $count;
    }
}
