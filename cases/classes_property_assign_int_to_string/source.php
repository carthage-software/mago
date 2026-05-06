<?php

declare(strict_types=1);

final class ClassesPropIntToStr
{
    public string $value = '';
}

function classesPropIntToStr(): void
{
    $obj = new ClassesPropIntToStr();
    $obj->value = 5;
}
