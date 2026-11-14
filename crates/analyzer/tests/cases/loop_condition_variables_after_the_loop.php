<?php

declare(strict_types=1);

final class Slot {}

function next_int(): int
{
    return rand(0, 3);
}

function acquire(): Slot|false
{
    return rand(0, 1) ? new Slot() : false;
}

function while_break(): int
{
    while ($value = next_int()) {
        if (rand(0, 1)) {
            break;
        }
    }

    return $value;
}

function for_break(): int
{
    for (; $value = next_int();) {
        if (rand(0, 1)) {
            break;
        }
    }

    return $value;
}

function right_of_and(): int
{
    while (rand(0, 1) && $value = next_int()) {
        echo 1;
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $value;
}

function exit_or_break(): Slot|false
{
    while (!($other = acquire())) {
        if (rand(0, 1)) {
            break;
        }
    }

    return $other;
}

/** @return 0 */
function last_for_condition_decides(): int
{
    for (; $w = 0, rand(0, 1);) {
        echo 1;
    }

    return $w;
}

function last_assignment_wins(): bool
{
    for (; $v = 42, $v = rand(0, 1) === 1;) {
        break;
    }

    return $v;
}
