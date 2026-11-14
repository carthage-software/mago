<?php

declare(strict_types=1);

function pre_increment(): string
{
    $try = 0;
    while (++$try < 10) {
        if (rand(0, 1)) {
            $name = 'thing';
        }
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    return $name;
}

function counted_for(): string
{
    for ($i = 0; $i < 10; $i++) {
        if (rand(0, 1)) {
            $name = 'thing';
        }
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    return $name;
}

function always_assigned(): string
{
    for ($i = 0; $i < 10; $i++) {
        $name = 'thing';
    }

    return $name;
}
