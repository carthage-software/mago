<?php

declare(strict_types=1);

function takesFloat(float $f): float
{
    return $f;
}

takesFloat(0.0);
takesFloat(-0.0);
takesFloat(1.5);
takesFloat(-3.14);
takesFloat(1e10);
takesFloat(1.5e-3);
takesFloat(.5);
takesFloat(5.);
