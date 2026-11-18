<?php

declare(strict_types=1);

final class FailureA extends Exception {}

final class FailureB extends Exception {}

/**
 * @template TFailure of Exception
 * @implements Iterator<int, string>
 */
final class GenericExceptionIterator implements Iterator
{
    /** @var TFailure */
    private Exception $failure;

    /** @param TFailure $failure */
    public function __construct(Exception $failure)
    {
        $this->failure = $failure;
    }

    /**
     * @return string
     * @throws TFailure
     */
    public function current(): mixed
    {
        throw $this->failure;
    }

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
final class ExceptionIteratorAggregate implements IteratorAggregate
{
    /** @var GenericExceptionIterator<FailureA>|GenericExceptionIterator<FailureB> */
    private Iterator $iterator;

    /**
     * @param GenericExceptionIterator<FailureA>|GenericExceptionIterator<FailureB> $iterator
     */
    public function __construct(Iterator $iterator)
    {
        $this->iterator = $iterator;
    }

    /** @return GenericExceptionIterator<FailureA>|GenericExceptionIterator<FailureB> */
    public function getIterator(): Traversable
    {
        return $this->iterator;
    }
}

/**
 * @throws Exception|FailureA|FailureB
 */
function iterate_exception_aggregate(ExceptionIteratorAggregate $aggregate): void
{
    foreach ($aggregate as $_value) {
    }
}

/**
 * @template TFailure of Exception
 * @implements IteratorAggregate<int, string>
 */
final class RecursiveExceptionIteratorAggregate implements IteratorAggregate
{
    /** @var TFailure */
    private Exception $failure;

    /** @param TFailure $failure */
    public function __construct(Exception $failure)
    {
        $this->failure = $failure;
    }

    /** @return GenericExceptionIterator<TFailure>|RecursiveExceptionIteratorAggregate<FailureB> */
    public function getIterator(): Traversable
    {
        return new GenericExceptionIterator($this->failure);
    }
}

/**
 * @param RecursiveExceptionIteratorAggregate<FailureA> $aggregate
 * @throws Exception|FailureA|FailureB
 */
function iterate_recursive_exception_aggregate(RecursiveExceptionIteratorAggregate $aggregate): void
{
    foreach ($aggregate as $_value) {
    }
}
