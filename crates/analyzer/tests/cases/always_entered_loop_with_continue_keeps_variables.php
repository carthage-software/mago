<?php

declare(strict_types=1);

function take_int(int $value): void
{
}

function do_while_assigned_before_continue(): void
{
    do {
        if (mt_rand(0, 1) === 1) {
            $value = 1;
            continue;
        }

        $value = 2;
    } while (mt_rand(0, 1) === 1);

    take_int($value);
}

function foreach_assigned_before_continue(): void
{
    foreach ([1, 2] as $_) {
        $value = 1;
        if (mt_rand(0, 1) === 1) {
            continue;
        }
    }

    take_int($value);
}

function do_while_skipped_by_continue(): void
{
    do {
        if (mt_rand(0, 1) === 1) {
            continue;
        }

        $value = 1;
    } while (mt_rand(0, 1) === 1);

    /** @mago-expect analysis:possibly-undefined-variable */
    take_int($value);
}

function for_skipped_by_continue(): void
{
    for ($i = 0; $i < 3; $i++) {
        if (mt_rand(0, 1) === 1) {
            continue;
        }

        $value = 1;
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    take_int($value);
}
