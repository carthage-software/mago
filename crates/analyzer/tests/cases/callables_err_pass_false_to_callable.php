<?php

declare(strict_types=1);

function callables_run_cb_more(callable $cb): mixed
{
    return $cb();
}

/** @mago-expect analysis:false-argument */
callables_run_cb_more(false);
