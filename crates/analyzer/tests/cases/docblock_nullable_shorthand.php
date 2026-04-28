<?php

declare(strict_types=1);

/**
 * @param ?int $x
 */
function takeNullableX(?int $x): int
{
    return $x ?? 0;
}

echo takeNullableX(null);
echo takeNullableX(5);
