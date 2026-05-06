<?php

declare(strict_types=1);

/** @param true $x */
function onlyTrueAD(bool $x): bool
{
    return $x;
}

onlyTrueAD(false);
