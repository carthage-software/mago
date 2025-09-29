<?php

/**
 * @return non-negative-int
 *
 * @mago-expect analysis:redundant-docblock-type
 */
function example(string $str): int
{
    /** @var non-negative-int $length */
    $length = strlen($str);

    return $length;
}
