<?php

declare(strict_types=1);

/** @param callable-string $c */
function takes_callable_string(string $c): void
{
    echo $c;
}

function probe(string $s): void
{
    takes_callable_string($s);
}
