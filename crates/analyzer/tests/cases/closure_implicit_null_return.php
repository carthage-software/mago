<?php

declare(strict_types=1);

function closure_implicit_null_return(int $x): void
{
    $falls_off = function (int $x) {
        if ($x > 0) {
            return 'a';
        }
    };

    $bare = function (int $x) {
        if ($x > 0) {
            return 'a';
        }

        return;
    };

    $always = function (int $x) {
        if ($x > 0) {
            return 'a';
        } else {
            return 'b';
        }
    };

    $arrow = fn(int $x) => $x > 0 ? 'a' : 'b';

    /** @mago-expect analysis:possibly-null-argument */
    echo strlen($falls_off($x));
    /** @mago-expect analysis:possibly-null-argument */
    echo strlen($bare($x));
    echo strlen($always($x));
    echo strlen($arrow($x));
}
