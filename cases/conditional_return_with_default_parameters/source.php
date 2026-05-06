<?php

declare(strict_types=1);

/**
 * @return list<int>
 */
function test_range_int_int(): array
{
    return range(1, 10);
}

/**
 * @return list<float>
 */
function test_float_step(): array
{
    return range(1, 10, 0.5);
}

/**
 * @return list<float>
 */
function test_float_start(): array
{
    return range(1.5, 10);
}

/**
 * @return list<string>
 */
function test_string_args(): array
{
    return range('a', 'z');
}
