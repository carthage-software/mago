<?php

declare(strict_types=1);

enum ClassesEnumCasesEnum
{
    case A;
    case B;
    case C;
}

$cases = ClassesEnumCasesEnum::cases();
echo count($cases);
