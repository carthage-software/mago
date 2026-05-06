<?php

/**
 * @return non-negative-int
 *
 */
function example(string $str): int
{
    /** @var non-negative-int $length */
    $length = strlen($str);

    return $length;
}
