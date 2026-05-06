<?php

declare(strict_types=1);

interface Foo
{
    public function foo($foo);

    const FOO = '';
}

trait Bar
{
    public function bar($bar) {}

    const BAR = '';
}

class Baz
{
    public function baz($baz) {}

    const BAZ = '';
}

abstract class Qux
{
    public function qux($baz);

    const QUX = '';
}
