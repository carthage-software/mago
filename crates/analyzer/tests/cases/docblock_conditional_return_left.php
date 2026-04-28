<?php

declare(strict_types=1);

/**
 * @template T of bool
 *
 * @param T $flag
 *
 * @return ($flag is true ? int : string)
 */
function pickJ(bool $flag): int|string
{
    return $flag ? 1 : 'no';
}

$a = pickJ(true);
echo $a + 1;
