<?php

declare(strict_types=1);

function bad_float_index(float $f): mixed
{
    // @mago-expect analysis:invalid-array-access
    return $f[0];
}
