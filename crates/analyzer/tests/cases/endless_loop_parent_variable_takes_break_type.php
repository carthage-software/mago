<?php

declare(strict_types=1);

function pick(): int
{
    $data = null;
    while (true) {
        if (rand(0, 1) === 1) {
            $data = 1;

            break;
        }
    }

    return $data;
}

function fetch(): int
{
    return rand(0, 10);
}

function retry(): int
{
    $data = null;
    while (true) {
        try {
            $data = fetch();

            break;
        } catch (RuntimeException) {
            usleep(10);
        }
    }

    return $data;
}
