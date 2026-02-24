<?php

namespace Fixture;

class Foo {}

/**
 * @psalm-type T = Foo
 */
class Bar
{
    /**
     * @var class-string<T>
     */
    const string KCLASS = Foo::class;

    /**
     * @return class-string<T>
     */
    public function bar(): string
    {
        return Foo::class;
    }
}
