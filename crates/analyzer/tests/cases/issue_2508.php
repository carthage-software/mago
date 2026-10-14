<?php

declare(strict_types=1);

namespace Issue2508;

use ArrayObject;
use Generator;
use Iterator;
use IteratorAggregate;
use Traversable;

/**
 * @phpstan-assert-if-true iterable<mixed, string> $value
 */
function all_strings(mixed $value): bool
{
    return is_iterable($value);
}

/**
 * @phpstan-assert-if-true iterable<mixed, int> $value
 */
function all_ints(mixed $value): bool
{
    return is_iterable($value);
}

/**
 * @template V
 * @template K of array-key
 *
 * @implements IteratorAggregate<K, V>
 */
final class Reversed implements IteratorAggregate
{
    /**
     * @param array<K, V> $items
     */
    public function __construct(
        private array $items,
    ) {}

    /**
     * @return Iterator<K, V>
     */
    public function getIterator(): Iterator
    {
        yield from $this->items;
    }
}

/**
 * @implements IteratorAggregate<int, string>
 */
final class Strings implements IteratorAggregate
{
    /**
     * @return Iterator<int, string>
     */
    public function getIterator(): Iterator
    {
        yield 'a';
    }
}

/**
 * @param iterable<mixed, string> $values
 */
function takes_strings(iterable $values): void
{
    foreach ($values as $value) {
        echo $value;
    }
}

/**
 * @param ArrayObject<array-key, string> $values
 */
function takes_string_array_object(ArrayObject $values): void
{
    takes_strings($values);
}

/**
 * @param Reversed<string, int> $values
 */
function takes_reversed_strings(Reversed $values): void
{
    takes_strings($values);
}

/**
 * @param array<array-key, mixed> $values
 */
function negated_on_mixed_array(array $values): int
{
    if (empty($values) || !all_strings($values)) {
        return 0;
    }

    takes_strings($values);

    return count($values);
}

/**
 * @param list<mixed> $values
 */
function negated_on_mixed_list(array $values, bool $flag): bool
{
    return $flag || !all_strings($values);
}

/**
 * @param Traversable<mixed, mixed> $values
 */
function negated_on_mixed_traversable(Traversable $values, bool $flag): void
{
    if ($flag || !all_strings($values)) {
        return;
    }

    takes_strings($values);
}

/**
 * @param ArrayObject<array-key, mixed> $values
 */
function negated_on_mixed_array_object(ArrayObject $values, bool $flag): int
{
    if ($flag || !all_strings($values)) {
        return 0;
    }

    takes_string_array_object($values);

    return count($values);
}

/**
 * @param iterable<mixed, mixed> $values
 */
function negated_on_mixed_iterable(iterable $values, bool $flag): bool
{
    return $flag || !all_strings($values);
}

/**
 * @param Generator<int, mixed, mixed, void> $values
 */
function narrows_generator(Generator $values): void
{
    if (!all_strings($values)) {
        return;
    }

    takes_strings($values);
}

/**
 * @param Iterator<int, mixed> $values
 */
function narrows_iterator(Iterator $values): void
{
    if (!all_strings($values)) {
        return;
    }

    takes_strings($values);
}

function narrows_bare_array_object(ArrayObject $values): void
{
    if (!all_strings($values)) {
        return;
    }

    takes_string_array_object($values);
}

/**
 * @param Reversed<mixed, int> $values
 */
function narrows_through_template_mapping(Reversed $values): void
{
    if (!all_strings($values)) {
        return;
    }

    takes_reversed_strings($values);
}

/**
 * @param non-empty-list<int> $values
 */
function negated_on_non_empty_int_list(array $values, bool $flag): bool
{
    /** @mago-expect analysis:impossible-type-comparison */
    return $flag || !all_strings($values);
}

/**
 * @param array<array-key, int> $values
 */
function negated_on_int_array(array $values, bool $flag): bool
{
    /** @mago-expect analysis:impossible-type-comparison */
    return $flag || !all_strings($values);
}

/**
 * @param Traversable<mixed, int> $values
 */
function negated_on_int_traversable(Traversable $values, bool $flag): bool
{
    /** @mago-expect analysis:impossible-type-comparison */
    return $flag || !all_strings($values);
}

/**
 * @param ArrayObject<array-key, int> $values
 */
function negated_on_int_array_object(ArrayObject $values, bool $flag): bool
{
    /** @mago-expect analysis:impossible-type-comparison */
    return $flag || !all_strings($values);
}

function negated_on_concrete_string_aggregate(Strings $values, bool $flag): bool
{
    /** @mago-expect analysis:impossible-type-comparison */
    return $flag || !all_ints($values);
}
