<?php

/** @param '23...'|'3...'|'...' $v */
function y(string $v): void
{
    echo "y: $v\n";
}

function example_switch(int $val): void
{
    $result = '';

    switch ($val) {
        case 2:
            $result .= '2';
        case 3:
            $result .= '3';
        default:
            $result .= '...';
    }

    y($result);
}
