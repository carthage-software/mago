<?php

/**
 * @mago-expect analysis:reference-constraint-violation
 */
function foo(string &$string): void
{
    $string = [];
}

/**
 * @param-out int $int
 *
 * @mago-expect analysis:reference-constraint-violation
 */
function bar(mixed &$int): void
{
    $int = [];
}
