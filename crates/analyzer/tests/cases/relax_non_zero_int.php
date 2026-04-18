<?php

/**
 * @param non-zero-int $n
 */
function takes_non_zero_int(int $n): void {}

/**
 * @return positive-int
 */
function pos(): int
{
    return 1;
}

/**
 * @return negative-int
 */
function neg(): int
{
    return -1;
}

function test_non_zero_int_valid(): void
{
    takes_non_zero_int(1);
    takes_non_zero_int(-1);
    takes_non_zero_int(pos());
    takes_non_zero_int(neg());
}

function test_non_zero_int_invalid(): void
{
    takes_non_zero_int(0); // @mago-expect analysis:invalid-argument
}
