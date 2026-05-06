<?php

declare(strict_types=1);

/** @param true $x */
function onlyTrueAC(bool $x): bool
{
    return $x;
}

/** @param false $x */
function onlyFalseAC(bool $x): bool
{
    return $x;
}

onlyTrueAC(true);
onlyFalseAC(false);
onlyTrueAC(false);
onlyFalseAC(true);
