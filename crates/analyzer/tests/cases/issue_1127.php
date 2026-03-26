<?php

declare(strict_types=1);

class Foo
{
    public static stdClass $foo;

    public static function get(): stdClass
    {
        return self::$foo;
    }
}

Closure::bind(
    function (): void {
        $x = new stdClass();
        $x->answer = 42;
        Foo::$foo = $x;
    },
    null,
    Foo::class,
)();
