<?php

declare(strict_types=1);

interface Foo
{
    // @mago-expect analysis:missing-return-type
    // @mago-expect analysis:missing-parameter-type
    public function foo($foo);

    // @mago-expect analysis:missing-constant-type
    const FOO = '';
}

trait Bar
{
    // @mago-expect analysis:missing-return-type
    // @mago-expect analysis:missing-parameter-type
    public function bar($bar) {}

    // @mago-expect analysis:missing-constant-type
    const BAR = '';
}

class Baz
{
    // @mago-expect analysis:missing-return-type
    // @mago-expect analysis:missing-parameter-type
    public function baz($baz) {}

    // @mago-expect analysis:missing-constant-type
    const BAZ = '';
}

abstract class Qux
{
    // @mago-expect analysis:missing-return-type
    // @mago-expect analysis:missing-parameter-type
    public function qux($baz);

    // @mago-expect analysis:missing-constant-type
    const QUX = '';
}
