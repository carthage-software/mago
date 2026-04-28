<?php

declare(strict_types=1);

function takesIntFloat(int|float $x): int|float { return $x; }

$a = 0 ** 0;
takesIntFloat($a);
