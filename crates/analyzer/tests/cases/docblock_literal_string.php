<?php

declare(strict_types=1);

/** @param 'foo'|'bar'|'baz' $s */
function takeLiteralY(string $s): string
{
    return $s;
}

echo takeLiteralY('foo');
echo takeLiteralY('bar');
/** @mago-expect analysis:invalid-argument */
echo takeLiteralY('qux');
