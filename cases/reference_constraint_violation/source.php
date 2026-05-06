<?php

/**
 */
function foo(string &$string): void
{
    $string = [];
}

/**
 * @param-out int $int
 *
 */
function bar(mixed &$int): void
{
    $int = [];
}
