<?php

declare(strict_types=1);

interface Foo {}

interface Bar {}

class K
{
    public int $id = 0;
}

/** @param Foo&Bar&object{id: int} $x */
function shape_last(object $x): int
{
    return $x->id;
}

/** @param Foo&Bar&K $x */
function named_last(object $x): int
{
    return $x->id;
}

/** @param object{id: int}&Foo $x */
function shape_first(object $x): int
{
    return $x->id;
}

/** @param Foo&object{id: int}&Bar $x */
function shape_middle(object $x): int
{
    return $x->id;
}
