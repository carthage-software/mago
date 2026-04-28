<?php

declare(strict_types=1);

/** @param numeric-string $s */
function takeNumericAP(string $s): int
{
    return (int) $s;
}

/** @mago-expect analysis:possibly-invalid-argument */
takeNumericAP('hello');
