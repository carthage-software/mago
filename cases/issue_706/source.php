<?php

/**
 * @return int<0, 65535>
 */
function get_int_in_range(): int
{
    return 44;
}

/**
 * @return positive-int
 */
function get_positive_int(): int
{
    return 44;
}

$first = get_int_in_range();
$second = get_positive_int();
if ($first === $second) {
    echo 'Equal!';
}
