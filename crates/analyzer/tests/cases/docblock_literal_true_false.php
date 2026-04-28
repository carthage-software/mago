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
/** @mago-expect analysis:false-argument */
onlyTrueAC(false);
/** @mago-expect analysis:invalid-argument */
onlyFalseAC(true);
