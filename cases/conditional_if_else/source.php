<?php

function get_string_or_null(): ?string
{
    return null;
}

/** @return 1 */
function i_take_string(string $_s): int
{
    return 1;
}

/** @return 2 */
function i_take_null(null $_s): int
{
    return 2;
}

/** @param 1|2 $_i */
function i_take_one_or_two(int $_i): void {}

function test(): void
{
    $value = get_string_or_null();
    $result = $value === null ? i_take_null($value) : i_take_string($value);
    i_take_one_or_two($result);
}
