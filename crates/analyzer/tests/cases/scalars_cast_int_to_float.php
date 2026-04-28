<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$x = 5;
takesFloat((float) $x);
takesFloat((float) 0);
takesFloat((float) PHP_INT_MAX);
