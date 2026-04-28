<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = 3 + 0.5;  // float 3.5
takesFloat($a);
