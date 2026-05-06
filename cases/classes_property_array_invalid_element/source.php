<?php

declare(strict_types=1);

final class ClassesPropArrInvalidElem
{
    /** @var list<int> */
    public array $items = [];
}

function classesPropArrAppendStr(ClassesPropArrInvalidElem $obj): void
{
    $obj->items[] = 'str';
}
