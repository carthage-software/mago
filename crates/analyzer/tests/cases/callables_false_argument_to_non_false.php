<?php

declare(strict_types=1);

function callables_needs_string_only(string $s): int
{
    return strlen($s);
}

/** @mago-expect analysis:false-argument */
callables_needs_string_only(false);
