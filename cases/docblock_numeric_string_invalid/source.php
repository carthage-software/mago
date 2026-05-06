<?php

declare(strict_types=1);

/** @param numeric-string $s */
function takeNumericAP(string $s): int
{
    return (int) $s;
}

takeNumericAP('hello');
