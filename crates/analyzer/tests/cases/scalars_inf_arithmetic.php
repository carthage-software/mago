<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = INF + 1;
takesFloat($a);
$b = INF - INF;
takesFloat($b);
$c = INF * 0.0;
takesFloat($c);
