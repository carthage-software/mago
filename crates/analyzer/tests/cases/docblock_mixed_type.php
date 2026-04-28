<?php

declare(strict_types=1);

/** @param mixed $x */
function takeMixedAT(mixed $x): bool
{
    return $x !== null;
}

takeMixedAT(1);
takeMixedAT(null);
takeMixedAT('a');
