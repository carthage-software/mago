<?php

declare(strict_types=1);

/**
 * @method int<0, max> doSomething()
 */
final class Example
{
    public function doSomething(): mixed
    {
        return 1;
    }
}

class Foo
{
}

/** @method Foo getFoo() */
class Bar
{
    /** @return object */
    function getFoo()
    {
        return new Foo();
    }
}

/**
 * @param int<0, max> $value
 */
function consume_integer(int $value): void
{
    echo $value;
}

function consume_foo(Foo $foo): void
{
    consume_foo($foo);
}

$example = new Example();
consume_integer($example->doSomething());

$bar = new Bar();
consume_foo($bar->getFoo());
