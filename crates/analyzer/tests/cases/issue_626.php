<?php

declare(strict_types=1);

class File
{
    /** @param iterable<self> $_values */
    public function foo(iterable $_values): void
    {
    }

    public function bar(): void
    {
        $this->foo(new ArrayIterator([]));
    }
}
