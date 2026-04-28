<?php

declare(strict_types=1);

function takesIntFloat(int|float $x): int|float {
    return $x * 2;
}

takesIntFloat(5);
takesIntFloat(2.5);
