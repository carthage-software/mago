<?php

declare(strict_types=1);

function callables_takes_closure_only(Closure $cb): mixed
{
    return $cb();
}

/** @mago-expect analysis:invalid-argument */
callables_takes_closure_only(42);
