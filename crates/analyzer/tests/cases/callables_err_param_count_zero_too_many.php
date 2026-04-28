<?php

declare(strict_types=1);

function callables_no_params(): int
{
    return 0;
}

/** @mago-expect analysis:too-many-arguments */
callables_no_params(1);
