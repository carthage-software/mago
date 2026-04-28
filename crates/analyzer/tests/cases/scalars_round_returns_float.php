<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = round(1.5);
takesFloat($a);
$b = floor(1.9);
takesFloat($b);
$c = ceil(1.1);
takesFloat($c);
