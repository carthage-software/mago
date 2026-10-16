<?php

declare(strict_types=1);

/**
 * @param "a"|"b" $value
 */
function accept_ab(string $value): void
{
    echo $value;
}

/**
 * @param string|bool $x
 * @param list<"a"|"b"|int> $y
 */
function test_strict_in_array(string|bool $x, array $y): void
{
    if (in_array($x, $y, true)) {
        accept_ab($x);
    }
}
