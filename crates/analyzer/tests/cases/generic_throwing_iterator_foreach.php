<?php

declare(strict_types=1);

/**
 * @template TFailure of Exception
 * @implements Iterator<int, string>
 */
final class GenericThrowingIterator implements Iterator
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
 * @template TFailure of Exception
 * @param GenericThrowingIterator<TFailure> $iterator
 * @throws TFailure
 */
function generic_throwing_iterator_foreach(GenericThrowingIterator $iterator): void
{
    foreach ($iterator as $_value) {
    }
}
