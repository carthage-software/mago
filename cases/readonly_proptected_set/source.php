<?php

class A
{
    public readonly string $foo;

    public function __construct()
    {
        $this->foo = 'initial';
    }

    /**
     */
    public function foo(): void
    {
        $this->foo = 'baz';
    }
}

class B extends A
{
    /**
     */
    public function foo(): void
    {
        $this->foo = 'qux';
    }
}

/**
 */
function example(): void
{
    $a = new A();
    $a->foo = 'bar';
}
