<?php

/**
 * @return 5
 *
 */
function conditional_with_assignment(): int
{
    $a = 0;
    return ($a = 5) ? $a : 2;
}
