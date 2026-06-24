<?php

declare(strict_types=1);

readonly class ReadonlyClassExample
{
    public function __construct(public int $prop = 1) {}
}

class ReadonlyPropertyExample
{
    public function __construct(public readonly int $prop = 1) {}
}

class HookedPropertyExample
{
    public int $prop {
        get => 1;
    }
}

class MutablePropertyExample
{
    public int $prop = 1;
}

$a = new ReadonlyClassExample();
/** @mago-expect analysis:invalid-unset */
unset($a->prop);

$b = new ReadonlyPropertyExample();
/** @mago-expect analysis:invalid-unset */
unset($b->prop);

$c = new HookedPropertyExample();
/** @mago-expect analysis:invalid-unset */
unset($c->prop);

$d = new MutablePropertyExample();
unset($d->prop);
