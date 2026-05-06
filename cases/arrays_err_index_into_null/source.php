<?php

declare(strict_types=1);

function bad_null_index(): mixed
{
    $x = null;
    return $x[0];
}
