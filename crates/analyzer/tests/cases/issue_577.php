<?php

declare(strict_types=1);

class Foo
{
    public function bar(): ?Bar
    {
        return null;
    }
    public function baz(): ?Baz
    {
        return null;
    }
}

class Bar
{
    public function f(): bool
    {
        return false;
    }

}

class Baz
{
    public function f(): bool
    {
        return false;
    }

}

$foo = new Foo();

if (null !== ($bar = $foo->bar()) && null !== ($baz = $foo->baz()) && $baz->f()) {
}
