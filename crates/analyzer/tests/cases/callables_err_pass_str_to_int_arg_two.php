<?php

declare(strict_types=1);

function callables_take_int_a(int $a): int
{
    return $a;
}

/** @mago-expect analysis:invalid-argument */
callables_take_int_a('not int');
