<?php

declare(strict_types=1);

function takesInt(int $n): int
{
    return $n;
}

takesInt(1_000);
takesInt(1_000_000);
takesInt(0xff_ff);
takesInt(0b1010_1010);
