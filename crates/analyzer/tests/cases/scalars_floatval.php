<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

takesFloat(floatval('3.14'));
takesFloat(floatval(42));
takesFloat(floatval('1.5e2'));
