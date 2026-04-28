<?php

declare(strict_types=1);

function takesIntFloat(int|float $x): int|float { return $x; }

$a = 2 ** 10;   // int 1024
takesIntFloat($a);
$b = 2 ** -1;   // float 0.5
takesIntFloat($b);
