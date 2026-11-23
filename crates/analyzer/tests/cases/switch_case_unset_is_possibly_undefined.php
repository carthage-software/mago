<?php

declare(strict_types=1);

function switch_unset_in_one_case(int $i): int
{
    $x = 1;
    switch ($i) {
        case 1:
            unset($x);
            break;
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    return $x;
}

function switch_unset_before_fallthrough(int $i): int
{
    $x = 1;
    switch ($i) {
        case 1:
            unset($x);
        case 2:
            echo 1;
            break;
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    return $x;
}

function switch_unset_then_reassigned(int $i): int
{
    $x = 1;
    switch ($i) {
        case 1:
            unset($x);
            $x = 2;
            break;
    }

    return $x;
}

function switch_unset_in_returning_case(int $i): int
{
    $x = 1;
    switch ($i) {
        case 1:
            unset($x);
            return 0;
    }

    return $x;
}
