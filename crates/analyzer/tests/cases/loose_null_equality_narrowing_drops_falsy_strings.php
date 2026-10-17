<?php

declare(strict_types=1);

function accept_string(string $_): void
{
}

/**
 * @mago-expect analysis:possibly-null-operand
 * @mago-expect analysis:null-operand
 */
function test_loose_null_equality_empty_string(?string $s): string
{
    if ($s == null) {
        if ($s === null) {
            return "hello";
        }

        accept_string($s);
    } else {
        accept_string($s);
    }

    return $s;
}

/**
 * @mago-expect analysis:possibly-null-operand
 * @mago-expect analysis:null-operand
 */
function test_loose_null_inequality_retains_string(?string $s): string
{
    if ($s != null) {
        accept_string($s);
        return $s;
    }

    return "default";
}

/**
 * @mago-expect analysis:possibly-null-operand
 * @mago-expect analysis:null-operand
 */
function test_negative_invalid_argument(?string $s): void
{
    if ($s == null) {
        if ($s === null) {
            return;
        }

        // @mago-expect analysis:invalid-argument
        accept_string(123);
    }
}
