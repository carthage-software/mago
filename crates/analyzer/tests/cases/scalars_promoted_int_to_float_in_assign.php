<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$x = 5.0;
$x += 1;
takesFloat($x);
