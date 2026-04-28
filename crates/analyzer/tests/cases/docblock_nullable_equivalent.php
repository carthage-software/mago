<?php

declare(strict_types=1);

/** @param ?int $x */
function shortNullable(?int $x): int
{
    return $x ?? 0;
}

/** @param int|null $y */
function longNullable(?int $y): int
{
    return shortNullable($y);
}

echo longNullable(null);
echo longNullable(7);
