<?php

declare(strict_types=1);

final class ClassesPropOnNull
{
    public int $value = 0;
}

function classesPropOnNull(): void
{
    $obj = null;
    /** @mago-expect analysis:null-property-access */
    $_ = $obj->value;
}
