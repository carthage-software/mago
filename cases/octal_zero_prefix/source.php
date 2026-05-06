<?php

/**
 * @param int<0, 600> $a
 */
function some_int(int $a): void
{
    echo $a;
}

some_int(0777); // ok
