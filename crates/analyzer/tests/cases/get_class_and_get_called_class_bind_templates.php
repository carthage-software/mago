<?php

declare(strict_types=1);

/**
 * @template T of object
 *
 * @param class-string<T> $className
 *
 * @return T|null
 */
function find(string $className): ?object
{
    return null;
}

class Foo
{
    public function hi(): string
    {
        return 'hi';
    }

    public function via_static(): string
    {
        return find(get_called_class())?->hi() ?? '';
    }
}

function via_get_class(Foo $foo): string
{
    return find(get_class($foo))?->hi() ?? '';
}

function via_class_constant(Foo $foo): string
{
    return find($foo::class)?->hi() ?? '';
}
