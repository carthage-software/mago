<?php

declare(strict_types=1);

/**
 * @template T of bool
 *
 * @param T $flag
 *
 * @return ($flag is not false ? int : string)
 */
function pickBX(bool $flag): int|string
{
    return $flag ? 1 : 'no';
}

$result = pickBX(true);
echo $result + 1;
