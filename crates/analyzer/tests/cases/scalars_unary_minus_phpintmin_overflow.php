<?php

declare(strict_types=1);

function takesIntFloat(int|float $x): int|float { return $x; }

$a = -PHP_INT_MIN;
takesIntFloat($a);
