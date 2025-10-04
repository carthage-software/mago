<?php

function i_take_non_bool_scalar(int|float|string $_val): void
{
}

function i_take_non_false_scalar(true|int|float|string $_val): void
{
}

function i_take_non_true_scalar(false|int|float|string $_val): void
{
}

function i_take_false(bool $_val): void
{
}

function i_take_true(bool $_val): void
{
}

/**
 * @param scalar $val
 */
function remove_false(mixed $val): void
{
    if ($val === false) {
        i_take_false($val);
        return;
    }

    i_take_non_false_scalar($val);
}

/**
 * @param scalar $val
 */
function remove_true(mixed $val): void
{
    if ($val === true) {
        i_take_true($val);
        return;
    }

    i_take_non_true_scalar($val);
}

/**
 * @param scalar $val
 */
function remove_true_and_false(mixed $val): void
{
    if ($val === true || $val === false) {
        return;
    }

    i_take_non_false_scalar($val);
    i_take_non_true_scalar($val);
    i_take_non_bool_scalar($val);
}
