<?php

declare(strict_types=1);

function empty_condition(): int
{
    for ($i = 0; ; $i++) {
        if ($i < 10) {
            continue;
        }

        return $i;
    }
}

function true_last_condition(): int
{
    for ($i = 0; $i < 5, true; $i++) {
        if (rand(0, 1) === 1) {
            return $i;
        }
    }
}

function bounded(): int // @mago-expect analysis:missing-return-statement
{
    for ($i = 0; $i < 5; $i++) {
        if (rand(0, 1) === 1) {
            return $i;
        }
    }
}
