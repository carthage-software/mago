<?php

declare(strict_types=1);

/** @return non-empty-list<int<1, 5>> */
function range_positive_ints(): array
{
    return range(1, 5);
}

/** @return non-empty-list<int<0, 5>> */
function range_non_negative_ints(): array
{
    return range(0, 5);
}

/** @return non-empty-list<int<-1, 5>> */
function range_mixed_sign_ints(): array
{
    return range(-1, 5);
}

/** @return non-empty-list<int<-1, 0>> */
function range_non_positive_ints(): array
{
    return range(-1, 0);
}

/** @return non-empty-list<int<-5, -1>> */
function range_negative_ints(): array
{
    return range(-5, -1);
}

/** @return non-empty-list<int<3, 8>> */
function range_reversed_ints(): array
{
    return range(8, 3);
}

/** @return non-empty-list<float> */
function range_positive_floats(): array
{
    return range(1.0, 5);
}

/** @return non-empty-list<float> */
function range_non_negative_floats(): array
{
    return range(0, 5.0);
}

/** @return non-empty-list<float> */
function range_mixed_floats(): array
{
    return range(-1.0, 5.0);
}

/** @return non-empty-list<non-empty-string> */
function range_strings(): array
{
    return range('a', 'z');
}

/** @return non-empty-list<non-empty-string> */
function range_digit_strings(): array
{
    return range('1', '5');
}

/** @return non-empty-list<int<0, 5>> */
function range_empty_string_to_int(): array
{
    return range('', 5);
}

/** @return non-empty-list<int<-1, 0>> */
function range_empty_string_from_negative(): array
{
    return range(-1, '');
}
