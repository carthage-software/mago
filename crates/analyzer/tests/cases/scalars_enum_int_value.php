<?php

declare(strict_types=1);

enum Status: int {
    case Active = 1;
    case Inactive = 0;
}

function takesInt(int $n): int { return $n; }

takesInt(Status::Active->value);
takesInt(Status::Inactive->value);
