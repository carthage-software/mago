<?php

declare(strict_types=1);

enum ClassesPureEnum
{
    case Active;
    case Inactive;
}

$status = ClassesPureEnum::Active;
echo $status->name;
