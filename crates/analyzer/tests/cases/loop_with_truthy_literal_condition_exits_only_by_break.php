<?php

declare(strict_types=1);

const ON = true;

function from_string(): string
{
    while ('yes') {
        if (rand(0, 1)) {
            $x = 's';
            break;
        }
    }

    return $x;
}

function from_negative_int(): string
{
    for ($i = 0; -1; $i++) {
        if (rand(0, 1)) {
            $x = 's';
            break;
        }
    }

    return $x;
}

function from_float(): string
{
    while (1.5) {
        if (rand(0, 1)) {
            $x = 's';
            break;
        }
    }

    return $x;
}

function from_constant(): string
{
    while (ON) {
        if (rand(0, 1)) {
            $x = 's';
            break;
        }
    }

    return $x;
}
