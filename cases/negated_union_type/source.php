<?php

/**
 * @param -1|-24.0|string $a
 */
function x(int|float|string $a): void
{
    echo $a;
}

x(-1);
x(-24.0);
x('hello');
