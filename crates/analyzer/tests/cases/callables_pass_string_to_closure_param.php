<?php

declare(strict_types=1);

function callables_needs_closure(Closure $cb): mixed
{
    return $cb();
}

/** @mago-expect analysis:invalid-argument */
callables_needs_closure('strlen');
