<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = 7 / 2;
takesFloat($a);
