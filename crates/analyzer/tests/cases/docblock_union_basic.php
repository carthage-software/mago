<?php

declare(strict_types=1);

/**
 * @param int|string $x
 */
function takeUnionW(int|string $x): string
{
    return (string) $x;
}

echo takeUnionW(1);
echo takeUnionW('a');
