<?php

namespace A;

class X implements SomeInterface
{
    public function __construct(private \B\SomeInterface $some) {}

    public function foo(): void
    {
        echo 'Hello';
    }
}
