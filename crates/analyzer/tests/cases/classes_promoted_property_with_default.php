<?php

declare(strict_types=1);

final class ClassesPromotedDefault
{
    public function __construct(public int $count = 0, public string $name = 'unset')
    {
    }
}

$obj = new ClassesPromotedDefault();
echo $obj->count;
echo $obj->name;
