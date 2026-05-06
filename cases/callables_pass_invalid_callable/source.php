<?php

declare(strict_types=1);

function callables_takes_callable_only(callable $cb): mixed
{
    return $cb(1);
}

callables_takes_callable_only(42);
