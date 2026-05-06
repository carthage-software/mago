<?php

declare(strict_types=1);

final class ClassesPropTypedArr
{
    /** @var list<int> */
    public array $items = [];
}

function classesPropTypedArr(ClassesPropTypedArr $obj): void
{
    $obj->items = 5;
}
