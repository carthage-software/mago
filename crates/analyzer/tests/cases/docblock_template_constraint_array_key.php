<?php

declare(strict_types=1);

/**
 * @template TK of array-key
 *
 * @param TK $k
 *
 * @return TK
 */
function passKeyBD(int|string $k): int|string
{
    return $k;
}

echo passKeyBD(1);
echo passKeyBD('a');
