<?php

declare(strict_types=1);

final class ClassesPropTypedArr
{
    /** @var list<int> */
    public array $items = [];
}

function classesPropTypedArr(ClassesPropTypedArr $obj): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    $obj->items = 5;
}
