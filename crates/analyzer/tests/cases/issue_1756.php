<?php

declare(strict_types=1);

final class Assert
{
    /**
     * @psalm-pure
     *
     * @psalm-template T of object
     *
     * @psalm-param class-string<T> $class
     *
     * @psalm-assert T $value
     *
     * @param mixed         $value
     * @param string|object $class
     * @param string        $message
     *
     * @return void
     *
     * @throws InvalidArgumentException
     */
    public static function isInstanceOf(mixed $value, object|string $class, string $message = ''): void
    {
        if (!$value instanceof $class) {
            throw new InvalidArgumentException($message);
        }
    }
}

final class Foo {}

function as_foo(object $foo): Foo
{
    Assert::isInstanceOf($foo, Foo::class);

    return $foo;
}
