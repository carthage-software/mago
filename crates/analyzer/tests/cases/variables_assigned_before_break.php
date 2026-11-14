<?php

declare(strict_types=1);

function only_exit(): int
{
    while (true) {
        if (rand(0, 1) === 1) {
            $found = 1;
            break;
        }
    }

    return $found;
}

function only_exit_for(): int
{
    for (;;) {
        if (rand(0, 1) === 1) {
            $found = 1;
            break;
        }
    }

    return $found;
}

/** @param list<int> $cells */
function maybe(array $cells): int
{
    foreach ($cells as $cell) {
        if ($cell === 1) {
            $found = $cell;
            break;
        }
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $found;
}

function one_break_without_it(): int
{
    while (true) {
        if (rand(0, 1) === 1) {
            $found = 1;
            break;
        }

        if (rand(0, 1) === 1) {
            break;
        }
    }

    // @mago-expect analysis:possibly-undefined-variable
    return $found;
}
