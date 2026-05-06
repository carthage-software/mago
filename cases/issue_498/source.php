<?php

/**
 */
function foo(array $array): void
{
    $token = array_shift($array);
    while ($token !== null) {
        $token = (string) $token;
        $token = array_shift($array);
    }
}
