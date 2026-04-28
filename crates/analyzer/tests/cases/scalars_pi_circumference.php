<?php

declare(strict_types=1);

function takesFloat(float $f): float { return $f; }

function circumference(float $radius): float {
    return 2.0 * M_PI * $radius;
}

takesFloat(circumference(5.0));
