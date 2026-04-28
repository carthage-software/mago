<?php

declare(strict_types=1);

/**
 * @template T of bool
 *
 * @param T $flag
 *
 * @return ($flag is not true ? string : int)
 */
function pickNegated(bool $flag): int|string
{
    return $flag ? 1 : 'no';
}

$a = pickNegated(true);
echo $a + 1;
