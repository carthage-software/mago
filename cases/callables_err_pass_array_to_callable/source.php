<?php

declare(strict_types=1);

function callables_run_callable(callable $cb): mixed
{
    return $cb();
}

callables_run_callable([1, 2, 3]);
