<?php

declare(strict_types=1);

function callables_run_cb_only(callable $cb): mixed
{
    return $cb();
}

/** @mago-expect analysis:null-argument */
callables_run_cb_only(null);
