<?php

declare(strict_types=1);

function callables_run_callable_arg(callable $cb): mixed
{
    return $cb();
}

/** @mago-expect analysis:invalid-argument */
callables_run_callable_arg(0);
