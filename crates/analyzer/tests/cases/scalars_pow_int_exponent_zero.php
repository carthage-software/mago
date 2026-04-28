<?php

declare(strict_types=1);

function takesIntFloat(int|float $x): int|float { return $x; }

$a = 5 ** 0;  // 1
takesIntFloat($a);
$b = 0 ** 5;  // 0
takesIntFloat($b);
