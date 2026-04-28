<?php

declare(strict_types=1);

enum ClassesEnumDupValues: string
{
    case Foo = 'a';
    /** @mago-expect analysis:duplicate-enum-case-value */
    case Bar = 'a';
}
