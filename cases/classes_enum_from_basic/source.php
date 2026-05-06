<?php

declare(strict_types=1);

enum ClassesEnumFromEnum: string
{
    case Yes = 'yes';
    case No = 'no';
}

$value = ClassesEnumFromEnum::from('yes');
echo $value->value;
