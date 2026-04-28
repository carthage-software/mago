<?php

declare(strict_types=1);

/** @param numeric-string $s */
function takeNumericAO(string $s): int
{
    return (int) $s;
}

echo takeNumericAO('42');
/** @mago-expect analysis:possibly-invalid-argument */
takeNumericAO('hello');
