<?php

declare(strict_types=1);

final class ClassesStaticPropAcc
{
    public static int $shared = 5;
}

echo ClassesStaticPropAcc::$shared;
ClassesStaticPropAcc::$shared = 6;
echo ClassesStaticPropAcc::$shared;
