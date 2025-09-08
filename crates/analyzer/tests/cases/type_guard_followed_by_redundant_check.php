<?php

function foo(string $_x): void
{
}

/**
 * @mago-expect analysis:redundant-condition - second if condition
 * @mago-expect analysis:redundant-logical-operation - `||` in second `if`
 * @mago-expect analysis:no-value - `$x` in `else if`
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
