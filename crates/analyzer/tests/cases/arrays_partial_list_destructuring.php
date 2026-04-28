<?php

declare(strict_types=1);

/**
 * @param list{int, int, int} $xs
 */
function take_first_third(array $xs): int
{
    [$a, , $c] = $xs;
    return $a + $c;
}
