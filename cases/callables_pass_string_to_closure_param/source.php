<?php

declare(strict_types=1);

function callables_needs_closure(Closure $cb): mixed
{
    return $cb();
}

callables_needs_closure('strlen');
