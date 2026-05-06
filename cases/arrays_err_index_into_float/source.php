<?php

declare(strict_types=1);

function bad_float_index(float $f): mixed
{
    return $f[0];
}
