<?php

declare(strict_types=1);

namespace App;

const OWN_FLAG = 4;

/**
 * @param int-mask<E_ERROR, E_WARNING> $flags
 *
 * @return int<0, 3>
 */
function namespaced_mask(int $flags): int
{
    return $flags;
}

/**
 * @param int-mask<\E_NOTICE> $flags
 *
 * @return 0|8
 */
function qualified_mask(int $flags): int
{
    return $flags;
}

/**
 * @param int-mask-of<E_ERROR|E_WARNING> $flags
 *
 * @return int<0, 3>
 */
function mask_of_union(int $flags): int
{
    return $flags;
}

/**
 * @param E_ERROR|E_WARNING $level
 *
 * @return 1|2
 */
function union_of_constants(int $level): int
{
    return $level;
}

/**
 * @param int-mask<OWN_FLAG, 1> $flags
 *
 * @return 0|1|4|5
 */
function own_constant(int $flags): int
{
    return $flags;
}

/**
 * @return int-mask<E_ERROR>
 */
function wrong_flag(): int
{
    return E_WARNING; // @mago-expect analysis:invalid-return-statement
}

/**
 * @param int-mask<PREG_UNMATCHED_AS_NULL> $flags
 *
 * @return 512
 */
function or_flag(int $flags): int
{
    return $flags | PREG_UNMATCHED_AS_NULL;
}
