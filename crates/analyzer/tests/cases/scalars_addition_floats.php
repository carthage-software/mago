<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = 1.5 + 2.5;  // 4.0
takesFloat($a);
$b = 0.1 + 0.2;  // 0.30000000000000004
takesFloat($b);
