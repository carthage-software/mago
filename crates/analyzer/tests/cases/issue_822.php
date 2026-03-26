<?php

declare(strict_types=1);

class Foo
{
    protected static stdClass $bar;
    protected string $x = '';
}

$_ = function (): void {
    $obj = new stdClass();
    $obj->baz = 'qux';
    // @mago-expect analysis:invalid-property-read
    Foo::$bar = $obj;
};

$_ = function (): void {
    // @mago-expect analysis:undefined-variable,mixed-property-access
    $this->x = 'd';
};

(\Closure::bind(
    function (): void {
        $obj = new stdClass();
        $obj->baz = 'qux';
        Foo::$bar = $obj;
    },
    null,
    Foo::class,
) ?? throw new Error('failed to bind the closure'))();

(\Closure::bind(
    function (): void {
        $this->x = 'd';
    },
    new Foo(),
    Foo::class,
) ?? throw new Error('failed to bind the closure'))();

(\Closure::bind(fn(): string => $this->x, new Foo(), Foo::class) ?? throw new Error('failed to bind the closure'))();
