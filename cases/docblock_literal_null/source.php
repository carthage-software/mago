<?php

declare(strict_types=1);

/** @param null $x */
function onlyNullAE(mixed $x): mixed
{
    return $x;
}

onlyNullAE(null);
