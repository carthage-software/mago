<?php

declare(strict_types=1);

function callables_named_int(int $value): int
{
    return $value;
}

/** @mago-expect analysis:invalid-argument */
callables_named_int(value: 'oops');
