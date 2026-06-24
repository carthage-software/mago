<?php

declare(strict_types=1);

enum Foo
{
    case A;
    case B;
    case C;
}

final class Bar
{
    public Foo $foo = Foo::A;
}

function updateFoo(Bar $bar): void
{
    $bar->foo = Foo::B;
}

function repro(Bar $bar): Foo
{
    match ($bar->foo) {
        Foo::A => updateFoo($bar),
        Foo::B => updateFoo($bar),
        Foo::C => updateFoo($bar),
    };

    return match ($bar->foo) {
        Foo::A => Foo::B,
        Foo::B => Foo::C,
        Foo::C => Foo::A,
    };
}
