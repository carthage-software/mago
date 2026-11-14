<?php

declare(strict_types=1);

interface Foo {}

interface Bar
{
    public function bar(): int;
}

function viaInstanceof(Foo $obj): int
{
    $isBar = $obj instanceof Bar;
    assert($isBar);

    return $obj->bar();
}

function viaNull(?Bar $x): int
{
    $ok = $x !== null;
    assert($ok);

    return $x->bar();
}
