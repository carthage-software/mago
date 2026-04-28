<?php

declare(strict_types=1);

/** @param callable-string $c */
function takes_callable_string(string $c): void
{
    echo $c;
}

function probe(string $s): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_callable_string($s);
}
