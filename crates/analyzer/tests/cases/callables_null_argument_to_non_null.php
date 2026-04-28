<?php

declare(strict_types=1);

function callables_needs_string(string $s): int
{
    return strlen($s);
}

/** @mago-expect analysis:null-argument */
callables_needs_string(null);
