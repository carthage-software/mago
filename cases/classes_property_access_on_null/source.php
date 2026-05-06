<?php

declare(strict_types=1);

final class ClassesPropOnNull
{
    public int $value = 0;
}

function classesPropOnNull(): void
{
    $obj = null;
    $_ = $obj->value;
}
