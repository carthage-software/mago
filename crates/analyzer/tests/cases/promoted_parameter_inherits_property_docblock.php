<?php

declare(strict_types=1);

class ParentClass
{
    /** @var non-empty-string */
    public string $name = 'a';
}

final class ChildClass extends ParentClass
{
    public function __construct(
        public string $name,
    ) {}
}

function valid(): ChildClass
{
    return new ChildClass('name');
}

function invalid(): ChildClass
{
    /** @mago-expect analysis:invalid-argument */
    return new ChildClass('');
}
