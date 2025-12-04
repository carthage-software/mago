<?php

function take_int(int $value): void
{
}

function take_string(string $value): void
{
}

/**
 * @mago-expect analysis:missing-template-parameter
 */
final class SomeIterator implements Iterator
{
    #[Override]
    public function current(): string
    {
        return 'hello';
    }

    #[Override]
    public function key(): int
    {
        return 0;
    }

    #[Override]
    public function next(): void
    {
    }

    #[Override]
    public function valid(): bool
    {
        return true;
    }

    #[Override]
    public function rewind(): void
    {
    }
}

$iterator = new SomeIterator();

foreach ($iterator as $k => $v) {
    take_int($k);
    take_string($v);
}
