<?php

declare(strict_types=1);

/** @param true $x */
function onlyTrueAD(bool $x): bool
{
    return $x;
}

/** @mago-expect analysis:false-argument */
onlyTrueAD(false);
