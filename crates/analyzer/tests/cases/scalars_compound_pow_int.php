<?php

declare(strict_types=1);

function takesIntFloat(int|float $x): int|float { return $x; }

$x = 2;
$x **= 8;
takesIntFloat($x);
