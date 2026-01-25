<?php

declare(strict_types=1);

final class Foo
{
    public int $bar = 0;
}

final class TakesClosure
{
    // @mago-expect analysis:write-only-property
    private Closure $callback;

    public function __construct(Closure $callback)
    {
        $this->callback = $callback;
    }
}

$_ = new TakesClosure(
    /**
     * @param array{id: Foo} $data
     */
    function (array $data): Foo {
        $foo = $data['id'];
        $foo->bar = 1;
        return $foo;
    },
);
