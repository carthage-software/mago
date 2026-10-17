<?php

declare(strict_types=1);

function accept_string(string $_): void
{
}

/**
 * @mago-expect analysis:possibly-null-operand
 * @mago-expect analysis:null-operand
 */
function test_loose_null_equality_falsy_strings(?string $s): string
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
