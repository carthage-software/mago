<?php

declare(strict_types=1);

/** @param array-key $k */
function takeKeyAR(int|string $k): string
{
    return (string) $k;
}

echo takeKeyAR(1);
echo takeKeyAR('a');
