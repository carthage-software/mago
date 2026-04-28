<?php

declare(strict_types=1);

/** @param non-falsy-string $s */
function takeNonFalsyBH(string $s): string
{
    return $s;
}

/** @mago-expect analysis:possibly-invalid-argument */
takeNonFalsyBH('0');
