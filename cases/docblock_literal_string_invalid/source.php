<?php

declare(strict_types=1);

/** @param 'foo'|'bar' $s */
function takeLiteralZ(string $s): string
{
    return $s;
}

echo takeLiteralZ('baz');
