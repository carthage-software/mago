<?php

declare(strict_types=1);

enum ClassesIntEnum: int
{
    case Low = 1;
    case High = 99;
}

echo ClassesIntEnum::Low->value + ClassesIntEnum::High->value;
