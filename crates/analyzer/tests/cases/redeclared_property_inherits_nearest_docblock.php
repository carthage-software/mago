<?php

declare(strict_types=1);

class GrandParentClass
{
    public $untyped = 'a';
}

class ParentClass extends GrandParentClass
{
    /** @var non-empty-string */
    public $untyped = 'b';
}

final class ChildClass extends ParentClass
{
    public $untyped = 'c';
}

/** @return non-empty-string */
function untyped(ChildClass $c): string
{
    return $c->untyped;
}
