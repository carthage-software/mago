<?php

declare(strict_types=1);

function takesFloat(float $f): float
{
    return $f;
}

takesFloat(M_PI);
takesFloat(M_E);
takesFloat(INF);
takesFloat(-INF);
takesFloat(NAN);
takesFloat(PHP_FLOAT_MAX);
takesFloat(PHP_FLOAT_EPSILON);
