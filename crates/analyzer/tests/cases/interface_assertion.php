<?php

/**
 * @template T
 */
interface Foo
{
    /**
     * @return T
     */
    public function get(): mixed;
}

interface Bar
{
}

/**
 * @template T
 *
 * @param class-string<T> $class
 *
 * @assert T $object
 *
 * @throws InvalidArgumentException
 */
function assert_instanceof(object $object, string $class): void
{
    if (!$object instanceof $class) {
        throw new InvalidArgumentException("Object is not an instance of {$class}");
    }
}

/**
 * @param Foo<string> $foo
 *
 * @return Bar
 *
 * @throws InvalidArgumentException
 */
function test(Foo $foo): Bar
{
    assert_instanceof($foo, Bar::class);

    return $foo;
}
