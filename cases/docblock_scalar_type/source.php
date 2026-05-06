<?php

declare(strict_types=1);

/** @param scalar $s */
function takeScalarAS(int|string|float|bool $s): string
{
    return (string) $s;
}

echo takeScalarAS(1);
echo takeScalarAS('a');
echo takeScalarAS(true);
echo takeScalarAS(1.5);
