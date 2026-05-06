<?php

declare(strict_types=1);

enum ClassesEnumWithConst
{
    public const string LABEL = 'enum';

    case Active;
}

echo ClassesEnumWithConst::LABEL;
