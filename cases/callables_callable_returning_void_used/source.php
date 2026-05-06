<?php

declare(strict_types=1);

/**
 * @param callable(): void $cb
 */
function callables_run_void(callable $cb): void
{
    $cb();
}

callables_run_void(function (): void {
    echo 'ran';
});
