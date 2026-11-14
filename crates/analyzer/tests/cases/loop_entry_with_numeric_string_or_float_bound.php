<?php

declare(strict_types=1);

function numeric_string_bound(): string
{
    $try = 0;
    while (++$try < '10') {
        $name = 'thing';
    }

    return $name;
}

function float_bound(): string
{
    $try = 0;
    while (++$try < 10.0) {
        $name = 'thing';
    }

    return $name;
}

function may_skip(): string
{
    $try = 0;
    while (++$try < '1') {
        $name = 'thing';
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    return $name;
}
