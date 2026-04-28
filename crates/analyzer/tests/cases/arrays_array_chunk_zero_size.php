<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 *
 * @return array<array-key, mixed>
 */
function array_chunk_zero(array $xs): array
{
    /** @mago-expect analysis:invalid-argument */
    return array_chunk($xs, 0);
}

/**
 * @param list<int> $xs
 *
 * @return array<array-key, mixed>
 */
function array_chunk_negative(array $xs): array
{
    /** @mago-expect analysis:invalid-argument */
    return array_chunk($xs, -3);
}
