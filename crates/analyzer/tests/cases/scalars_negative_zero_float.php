<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = -0.0;
takesFloat($a);
$b = 0.0 + -0.0;  // 0.0
takesFloat($b);
