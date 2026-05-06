<?php

declare(strict_types=1);

final class ClassesPropAssignNull
{
    public string $name = '';
}

function classesPropAssignNull(ClassesPropAssignNull $obj): void
{
    $obj->name = null;
}
