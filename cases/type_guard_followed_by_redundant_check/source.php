<?php

function foo(string $_x): void {}

/**
 */
function bar(string|null|false $x): void
{
    if (!is_string($x)) {
        return;
    }

    if (is_string($x) || is_string($x)) {
        if (is_string($x)) {
            foo($x);
        }
    } else if (is_string($x)) {
        echo 1;
    }
}
