<?php

declare(strict_types=1);

final class ClassesPropTypedStr
{
    public string $name = '';
}

function classesPropTypedStr(ClassesPropTypedStr $obj): void
{
    $obj->name = 0;
}
