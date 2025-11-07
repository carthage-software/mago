<?php

declare(strict_types=1);

class Foo
{
    /**
     * @param int $requestType bad type hint
     */
    public function __construct(
        private readonly null|int $requestType,
    ) {}
}

new Foo(null);
