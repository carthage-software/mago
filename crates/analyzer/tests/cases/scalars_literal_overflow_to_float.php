<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = 9223372036854775808;
takesFloat($a);
