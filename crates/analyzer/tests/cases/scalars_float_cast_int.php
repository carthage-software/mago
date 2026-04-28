<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

takesFloat((float) 0);
takesFloat((float) PHP_INT_MAX);
takesFloat((float) -1);
