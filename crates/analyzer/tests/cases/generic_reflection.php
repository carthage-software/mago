<?php

class Foo
{
    /**
     * @return 'Hello'
     */
    public function greet(): string
    {
        return 'Hello';
    }
}

/**
 * @template T
 */
final class Box
{
    /**
     * @param T $value
     */
    public function __construct(
        public mixed $value,
    ) {}
}

/**
 * @throws ReflectionException
 */
function one(Foo $foo): void
{
    $reflection = new ReflectionClass(Foo::class);
    $method = $reflection->getMethod('greet');
    $method_name = $method->getName();
    echo $foo->{$method_name}();
}

/**
 * @param Box<string> $handler
 *
 * @return 'Hello'|null
 *
 * @throws ReflectionException
 */
function two(Foo $foo, Box $handler): ?string
{
    $reflection = new ReflectionClass(Foo::class);
    if ($reflection->hasMethod($handler->value)) {
        return $foo->{$handler->value}();
    } else {
        return null;
    }
}

/**
 * @param Box<string> $handler
 *
 * @return 'Hello'|null
 *
 * @throws ReflectionException
 */
function three(Foo $foo, Box $handler): ?string
{
    $reflection = new ReflectionClass(Foo::class);
    foreach ($reflection->getMethods() as $method) {
        return $foo->{$method->getName()}();
    }

    return null;
}

/**
 * @param ReflectionClass<Foo> $foo
 * @param Box<string> $handler
 *
 * @return 'Hello'|null
 *
 * @throws ReflectionException
 */
function four(ReflectionClass $foo, Box $handler): ?string
{
    foreach ($foo->getMethods() as $method) {
        return $foo->getMethod($method->getName())->invoke(new Foo());
    }

    return null;
}
