<?php

// Test 1: Basic template substitution with @implements (MUST PASS)
// When Data implements Comparable<Data>, T should be substituted with Data
// So compare(mixed $other) with @param Data should be compatible with @param T
/**
 * @template T
 */
interface Comparable
{
    /**
     * @param T $other
     */
    public function compare(mixed $other): int;
}

/**
 * @implements Comparable<Data>
 */
class Data implements Comparable
{
    /** @param Data $other */
    public function compare(mixed $other): int
    {
        return -1;
    }
}

// Test 2: Template substitution with extends (MUST PASS)
/**
 * @template T
 */
abstract class BaseComparable
{
    /**
     * @param T $other
     * @return int
     */
    abstract public function compareTo(mixed $other): int;
}

/**
 * @extends BaseComparable<string>
 */
class StringComparable extends BaseComparable
{
    /** @param string $other */
    public function compareTo(mixed $other): int
    {
        return 0;
    }
}

// Test 3: Multiple templates (MUST PASS)
/**
 * @template TKey
 * @template TValue
 */
interface Mapper
{
    /**
     * @param TKey $key
     * @return TValue
     */
    public function map(mixed $key): mixed;
}

/**
 * @implements Mapper<int, string>
 */
class IntToStringMapper implements Mapper
{
    /**
     * @param int $key
     * @return string
     */
    public function map(mixed $key): mixed
    {
        return (string) $key;
    }
}

// Test 5: Template with return type (MUST PASS)
/**
 * @template T
 */
interface Factory
{
    /**
     * @return T
     */
    public function create(): mixed;
}

/**
 * @implements Factory<Data>
 */
class DataFactory implements Factory
{
    /** @return Data */
    public function create(): mixed
    {
        return new Data();
    }
}

// Test 6: Nested generic types (MUST PASS)
/**
 * @template T
 */
interface Container
{
    /**
     * @param array<T> $items
     */
    public function store(array $items): void;
}

/**
 * @implements Container<string>
 */
class StringContainer implements Container
{
    /** @param array<string> $items */
    public function store(array $items): void
    {
    }
}

// Test 7: Iterator<TKey, TValue> - Generic built-in interface (MUST PASS)
/**
 * @implements Iterator<int, string>
 */
class StringIterator implements Iterator
{
    /** @return string */
    public function current(): mixed
    {
        return '';
    }

    /** @return int */
    public function key(): mixed
    {
        return 0;
    }

    public function next(): void
    {
    }

    public function rewind(): void
    {
    }

    public function valid(): bool
    {
        return false;
    }
}

// Test 8: IteratorAggregate<TKey, TValue> - Generic built-in interface (MUST PASS)
/**
 * @implements IteratorAggregate<int, string>
 */
class StringCollection implements IteratorAggregate
{
    /** @return Iterator<int, string> */
    public function getIterator(): Iterator
    {
        return new StringIterator();
    }
}

// Test 9: ArrayAccess<TKey, TValue> - Generic built-in interface (MUST PASS)
/**
 * @implements ArrayAccess<string, int>
 */
class StringIntMap implements ArrayAccess
{
    /** @param string $offset */
    public function offsetExists(mixed $offset): bool
    {
        return false;
    }

    /**
     * @param string $offset
     * @return int
     */
    public function offsetGet(mixed $offset): mixed
    {
        return 0;
    }

    /**
     * @param string $offset
     * @param int $value
     */
    public function offsetSet(mixed $offset, mixed $value): void
    {
    }

    /** @param string $offset */
    public function offsetUnset(mixed $offset): void
    {
    }
}

// Test 10: Custom generic interface with nested Iterator (MUST PASS)
/**
 * @template T
 */
interface GenericIterable
{
    /** @return Iterator<int, T> */
    public function getIterator(): Iterator;
}

/**
 * @implements GenericIterable<string>
 */
class StringIterable implements GenericIterable
{
    /** @return Iterator<int, string> */
    public function getIterator(): Iterator
    {
        return new StringIterator();
    }
}

// Test 11: Multiple nested generics with Iterator (MUST PASS)
/**
 * @template T
 */
interface Collection
{
    /** @return Iterator<int, T> */
    public function iterate(): Iterator;

    /** @param array<int, T> $items */
    public function addAll(array $items): void;

    /** @return array<int, T> */
    public function toArray(): array;
}

/**
 * @implements Iterator<int, Data>
 */
class DataIterator implements Iterator
{
    /** @return Data */
    public function current(): mixed
    {
        return new Data();
    }

    /** @return int */
    public function key(): mixed
    {
        return 0;
    }

    public function next(): void
    {
    }

    public function rewind(): void
    {
    }

    public function valid(): bool
    {
        return false;
    }
}

/**
 * @implements Collection<Data>
 */
class DataCollection implements Collection
{
    /** @return Iterator<int, Data> */
    public function iterate(): Iterator
    {
        return new DataIterator();
    }

    /** @param array<int, Data> $items */
    public function addAll(array $items): void
    {
    }

    /** @return array<int, Data> */
    public function toArray(): array
    {
        return [];
    }
}

// Test 12: Iterable return type with generics (MUST PASS)
/**
 * @template T
 */
interface IterableProvider
{
    /** @return iterable<int, T> */
    public function provide(): iterable;
}

/**
 * @implements IterableProvider<string>
 */
class StringProvider implements IterableProvider
{
    /** @return iterable<int, string> */
    public function provide(): iterable
    {
        return [];
    }
}
