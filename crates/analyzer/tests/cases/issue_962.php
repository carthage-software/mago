<?php

declare(strict_types=1);

class Foo {}

/**
 * @template T of object
 */
interface ProviderInterface
{
    /**
     * @return T|iterable<T>|null
     */
    public function provide(): object|array|null;
}

/**
 * @implements ProviderInterface<Foo>
 */
final class FooProvider implements ProviderInterface
{
    public function provide(): ?Foo
    {
        return new Foo();
    }
}
