<?php

declare(strict_types=1);

enum ClassesEnumMethod: int
{
    case Low = 1;
    case High = 9;

    public function double(): int
    {
        return $this->value * 2;
    }
}

echo ClassesEnumMethod::High->double();
