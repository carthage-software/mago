<?php

declare(strict_types=1);

function callables_takes_closure_only(Closure $cb): mixed
{
    return $cb();
}

callables_takes_closure_only(42);
