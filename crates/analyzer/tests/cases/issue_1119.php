<?php declare(strict_types=1);

trait FooTrait
{
    private string $foo;

    public function test(): string
    {
        return $this->foo;
    }
}

class Foo
{
    use FooTrait;

    public function __construct(
        private string $foo,
    ) {}
}
