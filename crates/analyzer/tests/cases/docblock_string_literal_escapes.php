<?php

declare(strict_types=1);

/**
 * @param 'it\'s'|'a\\b' $s
 */
function single(string $s): void
{
    echo $s;
}

/**
 * @param "x\ny" $s
 */
function double(string $s): void
{
    echo $s;
}

single("it's");
single('a\\b');
double("x\ny");
