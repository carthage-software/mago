<?php

/** @param 'one' $one */
function take_one(string $one): void
{
    echo $one;
}

/** @param 'two' $two */
function take_two(string $two): void
{
    echo $two;
}

/** @param 'one'|'two' $type */
function take_one_or_two(string $type): void
{
    if ($type == 'one') {
        take_one($type);
    } else {
        take_two($type);
    }
}

function take_any(string $type): void
{
    if ($type == 'one') {
        take_one($type);
    } elseif ($type == 'two') {
        take_two($type);
    } else {
        echo 'default';
    }
}

function take_any_switch(string $type): void
{
    switch ($type) {
        case 'one':
            take_one($type);
            break;
        case 'two':
            take_two($type);
            break;
        default:
            echo 'default';
    }
}
