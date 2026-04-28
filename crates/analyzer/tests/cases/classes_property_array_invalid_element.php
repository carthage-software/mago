<?php

declare(strict_types=1);

final class ClassesPropArrInvalidElem
{
    /** @var list<int> */
    public array $items = [];
}

function classesPropArrAppendStr(ClassesPropArrInvalidElem $obj): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    $obj->items[] = 'str';
}
