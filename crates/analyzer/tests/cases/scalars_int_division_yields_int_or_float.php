<?php

declare(strict_types=1);

function takesIntOrFloat(int|float $x): int|float { return $x; }

$a = 6 / 2;
$b = 7 / 2;
takesIntOrFloat($a);
takesIntOrFloat($b);
