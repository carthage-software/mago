<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = 10 / 2.0;
takesFloat($a);
