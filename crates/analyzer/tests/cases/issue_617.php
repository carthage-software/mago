<?php

declare(strict_types=1);

class Foo
{
    /**
     * @param int $_requestType bad type hint
     */
    public function __construct(
        private readonly null|int $_requestType,
    ) {}
}

new Foo(null);
