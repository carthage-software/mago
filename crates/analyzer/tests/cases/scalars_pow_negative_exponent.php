<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

$a = 2 ** -1;
takesFloat($a);
