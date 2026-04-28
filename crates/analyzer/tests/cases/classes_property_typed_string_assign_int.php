<?php

declare(strict_types=1);

final class ClassesPropTypedStr
{
    public string $name = '';
}

function classesPropTypedStr(ClassesPropTypedStr $obj): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    $obj->name = 0;
}
