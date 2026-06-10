<?php

declare(strict_types=1);

/**
 * @param empty $value
 * @return true
 */
function takes_empty(mixed $value): bool
{
    return empty($value);
}

/**
 * @param empty-scalar $value
 * @return true
 */
function takes_empty_scalar(mixed $value): bool
{
    return empty($value);
}

function accepts_all_empty_values(): void
{
    takes_empty(null);
    takes_empty(0);
    takes_empty(0.0);
    takes_empty('0');
    takes_empty('');
    takes_empty(false);
    takes_empty([]);

    takes_empty_scalar(0);
    takes_empty_scalar(0.0);
    takes_empty_scalar('0');
    takes_empty_scalar('');
    takes_empty_scalar(false);
}

/**
 * @mago-expect analysis:invalid-argument
 */
function rejects_non_empty_int(): void
{
    takes_empty(1);
}

/**
 * @mago-expect analysis:invalid-argument
 */
function rejects_non_empty_string(): void
{
    takes_empty('a');
}

/**
 * @mago-expect analysis:invalid-argument
 */
function rejects_true(): void
{
    takes_empty(true);
}

/**
 * @mago-expect analysis:null-argument
 */
function rejects_null_for_empty_scalar(): void
{
    takes_empty_scalar(null);
}

/**
 * @mago-expect analysis:invalid-argument
 */
function rejects_array_for_empty_scalar(): void
{
    takes_empty_scalar([]);
}
