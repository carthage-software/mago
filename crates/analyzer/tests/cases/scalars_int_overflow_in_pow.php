<?php

declare(strict_types=1);

function takesIntFloat(int|float $x): int|float { return $x; }

$a = 10 ** 20;
takesIntFloat($a);
