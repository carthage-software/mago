<?php

namespace Ds;

use ArrayAccess;
use Countable;
use JsonSerializable;
use OutOfBoundsException;
use OutOfRangeException;
use Traversable;
use UnderflowException;

/**
 * @template-covariant TKey
 * @template-covariant TValue
 *
 * @extends Traversable<TKey, TValue>
 */
interface Collection extends Traversable, Countable, JsonSerializable
{
    /**
     * @return Collection<TKey, TValue>
     *
     * @mutation-free
     */
    public function copy(): Collection;

    /**
     * @return array<TKey, TValue>
     * @mutation-free
     */
    public function toArray(): array;

    /**
     * @mutation-free
     */
    public function isEmpty(): bool;

    /**
     * @mutation-free
     */
    public function count(): int;
}

/**
 * @template TValue
 * @implements Sequence<TValue>
 */
final class Deque implements Sequence
{
    /**
     * @param iterable<TValue> $values
     */
    public function __construct(iterable $values = []) {}

    /**
     * @return Deque<TValue>
     * @mutation-free
     */
    public function copy(): Deque
    {
    }

    /**
     * @return list<TValue>
     * @mutation-free
     */
    public function toArray(): array
    {
    }

    /**
     * @mutation-free
     */
    public function isEmpty(): bool
    {
    }

    /**
     * @mutation-free
     */
    public function count(): int
    {
    }

    /**
     * @mutation-free
     */
    public function allocate(int $capacity): void
    {
    }

    /**
     * @mutation-free
     */
    public function capacity(): int
    {
    }

    /**
     * @return float|int
     * @mutation-free
     */
    public function sum()
    {
    }

    /**
     * @param TValue ...$values
     * @mutation-free
     */
    public function contains(...$values): bool
    {
    }

    /**
     * @param (callable(TValue): bool)|null $callback
     *
     * @return Deque<TValue>
     *
     * @mutation-free
     */
    public function filter(callable $callback = null): Deque
    {
    }

    /**
     * @param TValue $value
     *
     * @return int|false
     *
     * @mutation-free
     */
    public function find($value)
    {
    }

    /**
     * @return TValue
     *
     * @throws UnderflowException
     *
     * @mutation-free
     */
    public function first()
    {
    }

    /**
     * @return TValue
     *
     * @throws OutOfRangeException
     *
     * @mutation-free
     */
    public function get(int $index)
    {
    }

    /**
     * @mutation-free
     */
    public function join(null|string $glue = null): string
    {
    }

    /**
     * @return TValue
     *
     * @throws UnderflowException
     *
     * @mutation-free
     */
    public function last()
    {
    }

    /**
     * @template TNewValue
     *
     * @param (callable(TValue): TNewValue) $callback
     *
     * @return Deque<TNewValue>
     *
     * @mutation-free
     */
    public function map(callable $callback): Deque
    {
    }

    /**
     * @template TValue2
     *
     * @param iterable<TValue2> $values
     *
     * @return Deque<TValue|TValue2>
     *
     * @mutation-free
     */
    public function merge(iterable $values): Deque
    {
    }

    /**
     * @return TValue
     *
     * @throws UnderflowException
     */
    public function pop()
    {
    }

    /**
     * @template TCarry
     *
     * @param (callable(TCarry, TValue): TCarry) $callback
     * @param TCarry $initial
     *
     * @return TCarry
     *
     * @mutation-free
     */
    public function reduce(callable $callback, $initial = null)
    {
    }

    /**
     * @return TValue
     * @throws \OutOfRangeException
     */
    public function remove(int $index)
    {
    }

    /**
     * @return Deque<TValue>
     * @mutation-free
     */
    public function reversed(): Deque
    {
    }

    /**
     * @return TValue
     * @throws \UnderflowException
     */
    public function shift()
    {
    }

    /**
     * @return Deque<TValue>
     * @mutation-free
     */
    public function slice(int $offset, null|int $length = null): Deque
    {
    }

    /**
     * @param (callable(TValue, TValue): int)|null $comparator
     * @return Deque<TValue>
     * @mutation-free
     */
    public function sorted(callable $comparator = null): Deque
    {
    }
}

/**
 * @template TKey
 * @template TValue
 * @implements Collection<TKey, TValue>
 * @implements ArrayAccess<TKey, TValue>
 */
final class Map implements Collection, ArrayAccess
{
    /**
     * @param iterable<TKey, TValue> $values
     */
    public function __construct(iterable $values = []) {}

    /**
     * @mutation-free
     */
    public function allocate(int $capacity): void
    {
    }

    /**
     * @mutation-free
     */
    public function capacity(): int
    {
    }

    /**
     * @return Map<TKey, TValue>
     * @mutation-free
     */
    public function copy(): Map
    {
    }

    /**
     * @return array<TKey, TValue>
     * @mutation-free
     */
    public function toArray(): array
    {
    }

    /**
     * @mutation-free
     */
    public function isEmpty(): bool
    {
    }

    /**
     * @mutation-free
     */
    public function count(): int
    {
    }

    /**
     * @param callable(TKey, TValue): TValue $callback
     */
    public function apply(callable $callback): void
    {
    }

    /**
     * @return Pair<TKey, TValue>
     * @throws UnderflowException
     * @mutation-free
     */
    public function first(): Pair
    {
    }

    /**
     * @return Pair<TKey, TValue>
     * @throws UnderflowException
     * @mutation-free
     */
    public function last(): Pair
    {
    }

    /**
     * @return Pair<TKey, TValue>
     * @throws OutOfRangeException
     * @mutation-free
     */
    public function skip(int $position): Pair
    {
    }

    /**
     * @template TKey2
     * @template TValue2
     * @param iterable<TKey2, TValue2> $values
     * @return Map<TKey|TKey2, TValue|TValue2>
     * @mutation-free
     */
    public function merge(iterable $values): Map
    {
    }

    /**
     * @template TKey2
     * @template TValue2
     * @param Map<TKey2, TValue2> $map
     * @return Map<TKey&TKey2, TValue>
     * @mutation-free
     */
    public function intersect(Map $map): Map
    {
    }

    /**
     * @template TValue2
     * @param Map<TKey, TValue2> $map
     * @return Map<TKey, TValue>
     * @mutation-free
     */
    public function diff(Map $map): Map
    {
    }

    /**
     * @param TKey $key
     * @mutation-free
     */
    public function hasKey($key): bool
    {
    }

    /**
     * @param TValue $value
     * @mutation-free
     */
    public function hasValue($value): bool
    {
    }

    /**
     * @param (callable(TKey, TValue): bool)|null $callback
     * @return Map<TKey, TValue>
     * @mutation-free
     */
    public function filter(callable $callback = null): Map
    {
    }

    /**
     * @template TDefault
     * @param TKey $key
     * @param TDefault $default
     * @return (
     *     func_num_args() is 1
     *     ? TValue
     *     : TValue|TDefault
     * )
     * @throws OutOfBoundsException
     * @mutation-free
     */
    public function get($key, $default = null)
    {
    }

    /**
     * @return Set<TKey>
     * @mutation-free
     */
    public function keys(): Set
    {
    }

    /**
     * @template TNewValue
     * @param callable(TKey, TValue): TNewValue $callback
     * @return Map<TKey, TNewValue>
     * @mutation-free
     */
    public function map(callable $callback): Map
    {
    }

    /**
     * @return Sequence<Pair<TKey, TValue>>
     * @mutation-free
     */
    public function pairs(): Sequence
    {
    }

    /**
     * @param TKey $key
     * @param TValue $value
     */
    public function put($key, $value)
    {
    }

    /**
     * @param iterable<TKey, TValue> $values
     */
    public function putAll(iterable $values)
    {
    }

    /**
     * @template TCarry
     * @param callable(TCarry, TKey, TValue): TCarry $callback
     * @param TCarry $initial
     * @return TCarry
     * @mutation-free
     */
    public function reduce(callable $callback, $initial = null)
    {
    }

    /**
     * @template TDefault
     * @param TKey $key
     * @param TDefault $default
     * @return (
     *     func_num_args() is 1
     *     ? TValue
     *     : TValue|TDefault
     * )
     * @throws \OutOfBoundsException
     */
    public function remove($key, $default = null)
    {
    }

    /**
     * @return Map<TKey, TValue>
     * @mutation-free
     */
    public function reversed(): Map
    {
    }

    /**
     * @return Map<TKey, TValue>
     * @mutation-free
     */
    public function slice(int $offset, null|int $length = null): Map
    {
    }

    /**
     * @param (callable(TValue, TValue): int)|null $comparator
     */
    public function sort(callable $comparator = null)
    {
    }

    /**
     * @param (callable(TValue, TValue): int)|null $comparator
     * @return Map<TKey, TValue>
     * @mutation-free
     */
    public function sorted(callable $comparator = null): Map
    {
    }

    /**
     * @param (callable(TKey, TKey): int)|null $comparator
     */
    public function ksort(callable $comparator = null)
    {
    }

    /**
     * @param (callable(TKey, TKey): int)|null $comparator
     * @return Map<TKey, TValue>
     * @mutation-free
     */
    public function ksorted(callable $comparator = null): Map
    {
    }

    /**
     * @return Sequence<TValue>
     * @mutation-free
     */
    public function values(): Sequence
    {
    }

    /**
     * @template TKey2
     * @template TValue2
     * @param Map<TKey2, TValue2> $map
     * @return Map<TKey|TKey2, TValue|TValue2>
     * @mutation-free
     */
    public function union(Map $map): Map
    {
    }

    /**
     * @template TKey2
     * @template TValue2
     * @param Map<TKey2, TValue2> $map
     * @return Map<TKey|TKey2, TValue|TValue2>
     * @mutation-free
     */
    public function xor(Map $map): Map
    {
    }
}

/**
 * @template-covariant TKey
 * @template-covariant TValue
 */
final class Pair implements JsonSerializable
{
    /**
     * @var TKey
     */
    public $key;

    /**
     * @var TValue
     */
    public $value;

    /**
     * @param TKey $key
     * @param TValue $value
     */
    public function __construct($key = null, $value = null) {}

    /**
     * @return Pair<TKey, TValue>
     * @mutation-free
     */
    public function copy(): Pair
    {
    }
}

/**
 * @template TValue
 * @extends Collection<int, TValue>
 * @extends ArrayAccess<int, TValue>
 */
interface Sequence extends Collection, ArrayAccess
{
    /**
     * @return Sequence<TValue>
     * @mutation-free
     */
    public function copy(): Sequence;

    /**
     * @return list<TValue>
     * @mutation-free
     */
    public function toArray(): array;

    /**
     * @mutation-free
     */
    public function isEmpty(): bool;

    /**
     * @mutation-free
     */
    public function count(): int;

    /**
     * @mutation-free
     */
    public function allocate(int $capacity): void;

    /**
     * @mutation-free
     */
    public function capacity(): int;

    /**
     * @return float|int
     * @mutation-free
     */
    public function sum();

    /**
     * @param callable(TValue): TValue $callback
     */
    public function apply(callable $callback): void;

    /**
     * @param TValue ...$values
     * @mutation-free
     */
    public function contains(...$values): bool;

    /**
     * @param (callable(TValue): bool)|null $callback
     * @return Sequence<TValue>
     * @mutation-free
     */
    public function filter(callable $callback = null): Sequence;

    /**
     * @param TValue $value
     * @return int|false
     * @mutation-free
     */
    public function find($value);

    /**
     * @return TValue
     * @throws \UnderflowException
     * @mutation-free
     */
    public function first();

    /**
     * @return TValue
     * @throws \OutOfRangeException
     * @mutation-free
     */
    public function get(int $index);

    /**
     * @param TValue ...$values
     * @throws \OutOfRangeException
     */
    public function insert(int $index, ...$values);

    /**
     * @mutation-free
     */
    public function join(null|string $glue = null): string;

    /**
     * @return TValue
     * @throws \UnderflowException
     * @mutation-free
     */
    public function last();

    /**
     * @template TNewValue
     * @param callable(TValue): TNewValue $callback
     * @return Sequence<TNewValue>
     * @mutation-free
     */
    public function map(callable $callback): Sequence;

    /**
     * @template TValue2
     * @param iterable<TValue2> $values
     * @return Sequence<TValue|TValue2>
     * @mutation-free
     */
    public function merge(iterable $values): Sequence;

    /**
     * @return TValue
     * @throws \UnderflowException
     */
    public function pop();

    /**
     * @param TValue ...$values
     */
    public function push(...$values);

    /**
     * @template TCarry
     * @param callable(TCarry, TValue): TCarry $callback
     * @param TCarry $initial
     * @return TCarry
     * @mutation-free
     */
    public function reduce(callable $callback, $initial = null);

    /**
     * @return TValue
     * @throws \OutOfRangeException
     */
    public function remove(int $index);

    /**
     * @return Sequence<TValue>
     * @mutation-free
     */
    public function reversed(): Sequence;

    /**
     * @param TValue $value
     * @throws \OutOfRangeException
     */
    public function set(int $index, $value);

    /**
     * @return TValue
     * @throws \UnderflowException
     */
    public function shift();

    /**
     * @return Sequence<TValue>
     * @mutation-free
     */
    public function slice(int $index, null|int $length = null): Sequence;

    /**
     * @param (callable(TValue, TValue): int)|null $comparator
     */
    public function sort(callable $comparator = null);

    /**
     * @param (callable(TValue, TValue): int)|null $comparator
     * @return Sequence<TValue>
     * @mutation-free
     */
    public function sorted(callable $comparator = null): Sequence;

    /**
     * @param TValue ...$values
     */
    public function unshift(...$values);
}

/**
 * @template TValue
 * @implements Sequence<TValue>
 */
final class Vector implements Sequence
{
    /**
     * @param iterable<TValue> $values
     */
    public function __construct(iterable $values = []) {}

    /**
     * @return Vector<TValue>
     * @mutation-free
     */
    public function copy(): Vector
    {
    }

    /**
     * @return list<TValue>
     * @mutation-free
     */
    public function toArray(): array
    {
    }

    /**
     * @mutation-free
     */
    public function isEmpty(): bool
    {
    }

    /**
     * @mutation-free
     */
    public function count(): int
    {
    }

    /**
     * @mutation-free
     */
    public function allocate(int $capacity): void
    {
    }

    /**
     * @mutation-free
     */
    public function capacity(): int
    {
    }

    /**
     * @return float|int
     * @mutation-free
     */
    public function sum()
    {
    }

    /**
     * @param TValue ...$values
     * @mutation-free
     */
    public function contains(...$values): bool
    {
    }

    /**
     * @param (callable(TValue): bool)|null $callback
     * @return Vector<TValue>
     * @mutation-free
     */
    public function filter(callable $callback = null): Vector
    {
    }

    /**
     * @param TValue $value
     * @return int|false
     * @mutation-free
     */
    public function find($value)
    {
    }

    /**
     * @return TValue
     * @throws \UnderflowException
     * @mutation-free
     */
    public function first()
    {
    }

    /**
     * @return TValue
     * @throws \OutOfRangeException
     * @mutation-free
     */
    public function get(int $index)
    {
    }

    /**
     * @mutation-free
     */
    public function join(null|string $glue = null): string
    {
    }

    /**
     * @return TValue
     * @throws \UnderflowException
     * @mutation-free
     */
    public function last()
    {
    }

    /**
     * @template TNewValue
     * @param callable(TValue): TNewValue $callback
     * @return Vector<TNewValue>
     * @mutation-free
     */
    public function map(callable $callback): Vector
    {
    }

    /**
     * @template TValue2
     * @param iterable<TValue2> $values
     * @return Vector<TValue|TValue2>
     * @mutation-free
     */
    public function merge(iterable $values): Sequence
    {
    }

    /**
     * @return TValue
     * @throws \UnderflowException
     */
    public function pop()
    {
    }

    /**
     * @template TCarry
     * @param callable(TCarry, TValue): TCarry $callback
     * @param TCarry $initial
     * @return TCarry
     * @mutation-free
     */
    public function reduce(callable $callback, $initial = null)
    {
    }

    /**
     * @return TValue
     * @throws \OutOfRangeException
     */
    public function remove(int $index)
    {
    }

    /**
     * @return Vector<TValue>
     * @mutation-free
     */
    public function reversed(): Vector
    {
    }

    /**
     * @return TValue
     * @throws \UnderflowException
     */
    public function shift()
    {
    }

    /**
     * @return Vector<TValue>
     * @mutation-free
     */
    public function slice(int $offset, null|int $length = null): Vector
    {
    }

    /**
     * @param (callable(TValue, TValue): int)|null $comparator
     * @return Vector<TValue>
     * @mutation-free
     */
    public function sorted(callable $comparator = null): Vector
    {
    }
}

/**
 * @template TValue
 * @implements Collection<int, TValue>
 * @implements ArrayAccess<int, TValue>
 */
final class Set implements Collection, ArrayAccess
{
    /**
     * @param iterable<TValue> $values
     */
    public function __construct(iterable $values = []) {}

    /**
     * @return Set<TValue>
     * @mutation-free
     */
    public function copy(): Set
    {
    }

    /**
     * @return list<TValue>
     * @mutation-free
     */
    public function toArray(): array
    {
    }

    /**
     * @mutation-free
     */
    public function isEmpty(): bool
    {
    }

    /**
     * @mutation-free
     */
    public function count(): int
    {
    }

    /**
     * @mutation-free
     */
    public function allocate(int $capacity): void
    {
    }

    /**
     * @mutation-free
     */
    public function capacity(): int
    {
    }

    /**
     * @param TValue ...$values
     */
    public function add(...$values): void
    {
    }

    /**
     * @param TValue ...$values
     * @mutation-free
     */
    public function contains(...$values): bool
    {
    }

    /**
     * @template TValue2
     * @param Set<TValue2> $set
     * @return Set<TValue>
     * @mutation-free
     */
    public function diff(Set $set): Set
    {
    }

    /**
     * @param (callable(TValue): bool)|null $callback
     * @return Set<TValue>
     * @mutation-free
     */
    public function filter(callable $callback = null): Set
    {
    }

    /**
     * @return TValue
     * @throws \UnderflowException
     * @mutation-free
     */
    public function first()
    {
    }

    /**
     * @return TValue
     * @throws \OutOfRangeException
     * @mutation-free
     */
    public function get(int $index)
    {
    }

    /**
     * @template TValue2
     * @param Set<TValue2> $set
     * @return Set<TValue&TValue2>
     * @mutation-free
     */
    public function intersect(Set $set): Set
    {
    }

    /**
     * @return TValue
     *
     * @throws \UnderflowException
     *
     * @mutation-free
     */
    public function last()
    {
    }

    /**
     * @template TNewValue
     *
     * @param callable(TValue): TNewValue $callback
     *
     * @return Set<TNewValue>
     */
    public function map(callable $callback): Set
    {
    }

    /**
     * @template TValue2
     *
     * @param iterable<TValue2> $values
     *
     * @return Set<TValue|TValue2>
     *
     * @mutation-free
     */
    public function merge(iterable $values): Set
    {
    }

    /**
     * @param TValue ...$values
     */
    public function remove(...$values): void
    {
    }

    /**
     * @return Set<TValue>
     *
     * @mutation-free
     */
    public function reversed(): Set
    {
    }

    /**
     * @return Set<TValue>
     *
     * @mutation-free
     */
    public function slice(int $index, null|int $length = null): Set
    {
    }

    /**
     * @param (callable(TValue, TValue): int)|null $comparator
     */
    public function sort(callable $comparator = null): void
    {
    }

    /**
     * @param (callable(TValue, TValue): int)|null $comparator
     *
     * @return Set<TValue>
     *
     * @mutation-free
     */
    public function sorted(callable $comparator = null): Set
    {
    }

    /**
     * @template TValue2
     *
     * @param Set<TValue2> $set
     *
     * @return Set<TValue|TValue2>
     *
     * @mutation-free
     */
    public function union(Set $set): Set
    {
    }

    /**
     * @template TValue2
     *
     * @param Set<TValue2> $set
     *
     * @return Set<TValue|TValue2>
     *
     * @mutation-free
     */
    public function xor(Set $set): Set
    {
    }
}

/**
 * @template TValue
 *
 * @implements Collection<int, TValue>
 * @implements ArrayAccess<int, TValue>
 */
final class Stack implements Collection, ArrayAccess
{
    /**
     * @param iterable<TValue> $values
     */
    public function __construct(iterable $values = []) {}

    /**
     * @return Stack<TValue>
     *
     * @mutation-free
     */
    public function copy(): Stack
    {
    }

    /**
     * @return list<TValue>
     *
     * @mutation-free
     */
    public function toArray(): array
    {
    }

    /**
     * @mutation-free
     */
    public function isEmpty(): bool
    {
    }

    /**
     * @mutation-free
     */
    public function count(): int
    {
    }

    /**
     * @mutation-free
     */
    public function allocate(int $capacity): void
    {
    }

    /**
     * @mutation-free
     */
    public function capacity(): int
    {
    }

    /**
     * @return TValue
     * @throws UnderflowException
     * @mutation-free
     */
    public function peek()
    {
    }

    /**
     * @return TValue
     * @throws UnderflowException
     */
    public function pop()
    {
    }

    /**
     * @param TValue ...$values
     */
    public function push(...$values): void
    {
    }
}

/**
 * @template TValue
 * @implements Collection<int, TValue>
 * @implements ArrayAccess<int, TValue>
 */
final class Queue implements Collection, ArrayAccess
{
    /**
     * @param iterable<TValue> $values
     */
    public function __construct(iterable $values = []) {}

    /**
     * @return Queue<TValue>
     * @mutation-free
     */
    public function copy(): Queue
    {
    }

    /**
     * @return list<TValue>
     * @mutation-free
     */
    public function toArray(): array
    {
    }

    /**
     * @mutation-free
     */
    public function isEmpty(): bool
    {
    }

    /**
     * @mutation-free
     */
    public function count(): int
    {
    }

    /**
     * @mutation-free
     */
    public function allocate(int $capacity): void
    {
    }

    /**
     * @mutation-free
     */
    public function capacity(): int
    {
    }

    /**
     * @return TValue
     *
     * @throws UnderflowException
     *
     * @mutation-free
     */
    public function peek()
    {
    }

    /**
     * @return TValue
     *
     * @throws UnderflowException
     */
    public function pop()
    {
    }

    /**
     * @param TValue ...$values
     */
    public function push(...$values): void
    {
    }
}

/**
 * @template TValue
 * @implements Collection<int, TValue>
 */
final class PriorityQueue implements Collection
{
    /**
     * @return PriorityQueue<TValue>
     *
     * @mutation-free
     */
    public function copy(): PriorityQueue
    {
    }

    /**
     * @return list<TValue>
     *
     * @mutation-free
     */
    public function toArray(): array
    {
    }

    /**
     * @mutation-free
     */
    public function isEmpty(): bool
    {
    }

    /**
     * @mutation-free
     */
    public function count(): int
    {
    }

    /**
     * @mutation-free
     */
    public function allocate(int $capacity): void
    {
    }

    /**
     * @mutation-free
     */
    public function capacity(): int
    {
    }

    /**
     * @return TValue
     *
     * @throws UnderflowException
     *
     * @mutation-free
     */
    public function peek()
    {
    }

    /**
     * @return TValue
     *
     * @throws UnderflowException
     */
    public function pop()
    {
    }

    /**
     * @param TValue $value
     */
    public function push($value, int $priority): void
    {
    }
}
