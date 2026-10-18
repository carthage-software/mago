<?php

class TooWideBaseFailure extends Exception
{
}

final class TooWideChildFailure extends TooWideBaseFailure
{
}

/**
 * @template T
 */
class TooWideGenericFailure extends Exception
{
    /** @param T $value */
    public function setValue(mixed $value): void
    {
    }
}

final class TooWideOtherFailure extends Exception
{
}

class TooWideIteratorFailure extends Exception
{
}

class TooWideGenericIteratorFailure extends Exception
{
}

/**
 * @implements Iterator<int, string>
 */
class TooWideThrowingIterator implements Iterator
{
    /**
     * @return string
     * @throws TooWideIteratorFailure
     */
    public function current(): mixed
    {
        throw new TooWideIteratorFailure();
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
 * @template TFailure of Throwable
 * @implements Iterator<int, string>
 */
class TooWideGenericThrowingIterator implements Iterator
{
    /** @param TFailure $failure */
    public function __construct(Throwable $failure)
    {
    }

    /**
     * @return string
     * @throws TFailure
     */
    public function current(): mixed
    {
        return 'value';
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

class TooWideKeyFailure extends Exception
{
}

/**
 * @implements Iterator<int, string>
 */
class TooWideKeyThrowingIterator implements Iterator
{
    /** @return string */
    public function current(): mixed
    {
        return 'value';
    }

    /**
     * @return int
     * @throws TooWideKeyFailure
     */
    public function key(): mixed
    {
        throw new TooWideKeyFailure();
    }

    public function next(): void
    {
    }

    public function rewind(): void
    {
    }

    public function valid(): bool
    {
        return true;
    }
}

class TooWideAggregateFailure extends Exception
{
}

class TooWideAggregateIteratorFailure extends Exception
{
}

/**
 * @implements Iterator<int, string>
 */
class TooWideAggregateIterator implements Iterator
{
    /**
     * @return string
     * @throws TooWideAggregateIteratorFailure
     */
    public function current(): mixed
    {
        throw new TooWideAggregateIteratorFailure();
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
 * @implements Iterator<int, string>
 */
class TooWideNonThrowingIterator implements Iterator
{
    /** @return string */
    public function current(): mixed
    {
        return 'value';
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
 * @implements IteratorAggregate<int, string>
 */
final class TooWideThrowingAggregate implements IteratorAggregate
{
    public function __construct(private bool $shouldThrow)
    {
    }

    /** @throws TooWideAggregateFailure */
    public function getIterator(): TooWideAggregateIterator
    {
        if ($this->shouldThrow) {
            throw new TooWideAggregateFailure();
        }

        return new TooWideAggregateIterator();
    }
}

/**
 * @implements IteratorAggregate<int, string>
 */
class TooWideInheritedThrowAggregate implements IteratorAggregate
{
    public function getIterator(): TooWideNonThrowingIterator
    {
        return new TooWideNonThrowingIterator();
    }
}

/**
 * @implements IteratorAggregate<int, string>
 */
class TooWideNestedAggregate implements IteratorAggregate
{
    /** @throws void */
    public function getIterator(): TooWideThrowingAggregate
    {
        return new TooWideThrowingAggregate(false);
    }
}

/**
 * @implements IteratorAggregate<int, string>
 */
class TooWideCyclicAggregate implements IteratorAggregate
{
    /** @throws void */
    public function getIterator(): TooWideCyclicAggregate
    {
        return $this;
    }
}

abstract class TooWideAbstractThrowContract
{
    /** @throws TooWideOtherFailure */
    abstract public function run(): void;
}

final class TooWideNarrowThrowImplementation extends TooWideAbstractThrowContract
{
    #[Override]
    public function run(): void
    {
    }
}

/**
 * @implements IteratorAggregate<int, string>
 */
final class TooWideFinalNonThrowingAggregate implements IteratorAggregate
{
    public function getIterator(): TooWideNonThrowingIterator
    {
        return new TooWideNonThrowingIterator();
    }
}

/** @implements IteratorAggregate<int, string> */
class TooWideBaseNonThrowingAggregate implements IteratorAggregate
{
    public function getIterator(): TooWideNonThrowingIterator
    {
        return new TooWideNonThrowingIterator();
    }
}

final class TooWideFinalInheritedNonThrowingAggregate extends TooWideBaseNonThrowingAggregate
{
}

/**
 * @throws TooWideChildFailure
 */
function too_wide_throws_child(): void
{
    throw new TooWideChildFailure();
}

/**
 * @throws TooWideBaseFailure
 */
function too_wide_throws_parent_contract(bool $shouldThrow): void
{
    if ($shouldThrow) {
        throw new TooWideChildFailure();
    }
}

/**
 * @param TooWideGenericFailure<int> $failure
 * @throws TooWideGenericFailure<int>
 */
function too_wide_throws_generic(TooWideGenericFailure $failure): void
{
    throw $failure;
}

/**
 * @throws TooWideChildFailure|TooWideOtherFailure
 */
function too_wide_throws_partial_union(): void
{
    throw new TooWideChildFailure();
}

/**
 * @throws TooWideChildFailure
 */
function too_wide_throws_narrow_contract(): void
{
    throw new TooWideBaseFailure();
}

/**
 * @throws TooWideChildFailure
 */
function too_wide_throws_rethrow(): void
{
    try {
        throw new TooWideChildFailure();
    } catch (TooWideChildFailure $failure) {
        throw $failure;
    }
}

/**
 * @throws TooWideChildFailure
 */
function too_wide_throws_from_match_arm(bool $shouldThrow): string
{
    return match ($shouldThrow) {
        true => throw new TooWideChildFailure(),
        false => 'continue',
    };
}

/**
 * @throws TooWideIteratorFailure
 */
function too_wide_throws_from_iterator(TooWideThrowingIterator $iterator): void
{
    foreach ($iterator as $value) {
    }
}

/**
 * @param TooWideGenericThrowingIterator<TooWideGenericIteratorFailure> $iterator
 * @throws TooWideGenericIteratorFailure
 */
function too_wide_throws_from_generic_iterator(TooWideGenericThrowingIterator $iterator): void
{
    foreach ($iterator as $value) {
    }
}

/**
 * @throws TooWideAggregateFailure|TooWideAggregateIteratorFailure
 */
function too_wide_throws_from_iterator_aggregate(TooWideThrowingAggregate $aggregate): void
{
    foreach ($aggregate as $value) {
    }
}

/** @throws Exception */
function too_wide_throws_from_iterator_aggregate_interface_contract(TooWideInheritedThrowAggregate $aggregate): void
{
    foreach ($aggregate as $value) {
    }
}

/**
 * @throws TooWideAggregateFailure|TooWideAggregateIteratorFailure
 */
function too_wide_throws_from_nested_iterator_aggregate(TooWideNestedAggregate $aggregate): void
{
    foreach ($aggregate as $value) {
    }
}

function too_wide_throws_from_cyclic_iterator_aggregate(TooWideCyclicAggregate $aggregate): void
{
    foreach ($aggregate as $value) {
    }
}

function too_wide_throws_from_final_non_throwing_iterator_aggregate(TooWideFinalNonThrowingAggregate $aggregate): void
{
    foreach ($aggregate as $value) {
    }
}

/** @throws Exception */
function too_wide_throws_from_final_inherited_non_throwing_iterator_aggregate(
    TooWideFinalInheritedNonThrowingAggregate $aggregate,
): void {
    foreach ($aggregate as $value) {
    }
}

/**
 * @throws TooWideKeyFailure
 */
function too_wide_throws_iterator_key_only_by_value(TooWideKeyThrowingIterator $iterator): void
{
    foreach ($iterator as $value) {
    }
}

/**
 * @throws TooWideKeyFailure
 */
function too_wide_throws_iterator_key_with_key(TooWideKeyThrowingIterator $iterator): void
{
    foreach ($iterator as $key => $value) {
    }
}

/**
 * @throws TooWideChildFailure
 */
function too_wide_throws_caught_only(): void
{
    try {
        throw new TooWideChildFailure();
    } catch (TooWideChildFailure) {
    }
}

/**
 * @throws TooWideOtherFailure
 */
function too_wide_throws_never(): void
{
}

/** @throws void */
function too_wide_throws_void(): void
{
}

/**
 * @throws TooWideChildFailure
 */
function too_wide_throws_call_child(): void
{
    too_wide_throws_child();
}

class TooWideThrowsRepository
{
    /**
     * @throws TooWideOtherFailure
     */
    public function unused(): void
    {
    }
}

class TooWideThrowsParent
{
    /** @throws TooWideChildFailure */
    public function inherited(): void
    {
        throw new TooWideChildFailure();
    }
}

class TooWideThrowsChild extends TooWideThrowsParent
{
    public function inherited(): void
    {
    }
}

/**
 * @throws TooWideOtherFailure
 */
function too_wide_throws_from_immediate_callback(): void
{
    array_any([1], /** @throws TooWideOtherFailure */ static fn(int $value): bool => throw new TooWideOtherFailure());
}

/**
 * @throws TooWideOtherFailure
 */
function too_wide_throws_from_named_callback(): void
{
    array_any(
        callback: /** @throws TooWideOtherFailure */ static fn(int $value): bool => throw new TooWideOtherFailure(),
        array: [1],
    );
}

/**
 * @throws TooWideOtherFailure
 */
function too_wide_throws_from_callback_variable(): void
{
    $comparator = /** @throws TooWideOtherFailure */ static function (int $left, int $right): int {
        throw new TooWideOtherFailure();
    };

    $values = [1, 2];
    usort($values, $comparator);
}

/**
 * @throws TooWideOtherFailure
 */
function too_wide_throws_from_array_map_callback(): void
{
    array_map(
        /** @throws TooWideOtherFailure */
        static function (mixed $value): int {
            throw new TooWideOtherFailure();
        },
        [1],
    );
}

/** @throws TooWideOtherFailure */
function too_wide_throws_from_preg_replace_callback_array(): void
{
    preg_replace_callback_array(
        [
            '/.*/' => /** @throws TooWideOtherFailure */ static function (array $matches): string {
                throw new TooWideOtherFailure();
            },
        ],
        'value',
    );
}
