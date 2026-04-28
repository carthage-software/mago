<?php

declare(strict_types=1);

/** @param 'foo'|'bar' $s */
function takeLiteralZ(string $s): string
{
    return $s;
}

/** @mago-expect analysis:invalid-argument */
echo takeLiteralZ('baz');
