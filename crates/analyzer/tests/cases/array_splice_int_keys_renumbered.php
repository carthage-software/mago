<?php

declare(strict_types=1);

/**
 * @param array<int, bool> $bools
 * @return list<bool>
 */
function array_splice_int_keys_removed(array $bools): array
{
    return array_splice($bools, 0, 1);
}

/**
 * @param array<int, bool> $bools
 * @return list<bool>
 */
function array_splice_int_keys_remaining(array $bools): array
{
    array_splice($bools, 0, 1);

    return $bools;
}
