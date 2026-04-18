<?php

/**
 * @return int<0, int>
 */
function from_zero_to_max(): int
{
    return 42;
}

/**
 * @return int<int, 0>
 */
function from_min_to_zero(): int
{
    return -5;
}

/**
 * @param non-negative-int $_n
 */
function takes_non_negative(int $_n): void {}

/**
 * @param non-positive-int $_n
 */
function takes_non_positive(int $_n): void {}

function test_int_range_unbounded_keyword(): void
{
    takes_non_negative(from_zero_to_max());
    takes_non_positive(from_min_to_zero());
}
