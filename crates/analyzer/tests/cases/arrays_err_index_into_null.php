<?php

declare(strict_types=1);

function bad_null_index(): mixed
{
    $x = null;
    // @mago-expect analysis:null-array-access
    return $x[0];
}
