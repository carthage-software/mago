<?php

/**
 * @psalm-type FooValueType = 1
 */
final class Foo
{
    const int FOO = 1;
}

/**
 * @psalm-type Bar = array{'baz': Baz}
 * @psalm-type Baz = array{'qux': Qux}
 * @psalm-type Qux = array{'foo': FooValueType}
 *
 * @psalm-import-type FooValueType from Foo
 */
class Bar
{
    /**
     * @param Bar $val
     *
     * @return 1
     */
    public function x(mixed $val): int
    {
        return $val['baz']['qux']['foo'];
    }
}
