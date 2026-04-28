<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = fmod(10.5, 3.0);
takesFloat($a);
$b = fmod(-7.5, 2.0);
takesFloat($b);
