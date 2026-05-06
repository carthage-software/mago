<?php

declare(strict_types=1);

enum ClassesEnumTryFromEnum: string
{
    case Yes = 'yes';
    case No = 'no';
}

$value = ClassesEnumTryFromEnum::tryFrom('maybe');
if (null !== $value) {
    echo $value->value;
}
