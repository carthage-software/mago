<?php

declare(strict_types=1);

main(function (): int {
    echo foo();

    return 0;
});

/**
 * @param (Closure(): int) $cb
 */
function main(Closure $cb): never
{
    $code = $cb();

    exit($code);
}

function foo(): string
{
    return 1; // @mago-expect analysis:invalid-return-statement
}
