<?php

declare(strict_types=1);

final class ClassesStaticPropAsgnInv
{
    public static int $count = 0;
}

function classesStaticPropAsgnInv(): void
{
    ClassesStaticPropAsgnInv::$count = [];
}
