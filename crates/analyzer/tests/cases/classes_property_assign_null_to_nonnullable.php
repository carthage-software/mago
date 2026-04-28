<?php

declare(strict_types=1);

final class ClassesPropAssignNull
{
    public string $name = '';
}

function classesPropAssignNull(ClassesPropAssignNull $obj): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    $obj->name = null;
}
