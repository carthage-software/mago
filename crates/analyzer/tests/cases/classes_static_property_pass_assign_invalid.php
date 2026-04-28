<?php

declare(strict_types=1);

final class ClassesStaticPropAsgnInv
{
    public static int $count = 0;
}

function classesStaticPropAsgnInv(): void
{
    /** @mago-expect analysis:invalid-property-assignment-value */
    ClassesStaticPropAsgnInv::$count = [];
}
