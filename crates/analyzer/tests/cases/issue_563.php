<?php

declare(strict_types=1);

/**
 * @template TKey of array-key
 * @template-covariant T
 *
 * @template-extends IteratorAggregate<TKey, T>
 */
interface ReadableCollection extends IteratorAggregate
{
    /**
     * Gets all values of the collection.
     *
     * @return mixed[] The values of all elements in the collection, in the
     *                 order they appear in the collection.
     *
     * @psalm-return list<T>
     */
    public function getValues();
}

/**
 * @template TKey of array-key
 * @template T
 * @template-extends ReadableCollection<TKey, T>
 */
interface Collection extends ReadableCollection
{
}

class Task
{
}

/**
 * @param list<Task>|Collection<int, Task> $tasks
 *
 * @return list<Task>
 */
function tasks(iterable $tasks): array
{
    if ($tasks instanceof Collection) {
        $tasks = $tasks->getValues();
    }

    return $tasks;
}

/**
 * @param list<Task>|Collection<int, Task> $tasks
 *
 * @return list<Task>
 */
function tasks_return(iterable $tasks): array
{
    if ($tasks instanceof Collection) {
        return $tasks->getValues();
    } else {
        return $tasks;
    }
}

/**
 * @param list<Task>|Collection<int, Task> $tasks
 *
 * @return list<Task>
 */
function tasks_conditional(iterable $tasks): array
{
    return $tasks instanceof Collection ? $tasks->getValues() : $tasks;
}

/**
 * @param list<Task>|Collection<int, Task> $tasks
 *
 * @return list<Task>
 */
function tasks_assign(iterable $tasks): array
{
    $tasks = $tasks instanceof Collection ? $tasks->getValues() : $tasks;

    return $tasks;
}
