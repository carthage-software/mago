<?php

declare(strict_types=1);

final class ClassesPropIntToStr
{
    public string $value = '';
}

function classesPropIntToStr(): void
{
    $obj = new ClassesPropIntToStr();
    /** @mago-expect analysis:invalid-property-assignment-value */
    $obj->value = 5;
}
