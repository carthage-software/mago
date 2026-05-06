<?php

declare(strict_types=1);

enum ClassesStringEnum: string
{
    case Active = 'active';
    case Inactive = 'inactive';
}

$status = ClassesStringEnum::Active;
echo $status->value;
