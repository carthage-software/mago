<?php

declare(strict_types=1);

function broken_example(): bool
{
    $labels = [];

    for ($i = 0; $i < 10; $i++) {
        $labels[$i] = 0;
    }

    for ($i = 0; $i < 10; $i++) {
        if ($labels[$i] === 0 && do_stuff($labels)) {
            return false;
        }
    }

    return true;
}

function working_example(): bool
{
    $labels = [];

    for ($i = 0; $i < 10; $i++) {
        $labels[$i] = 0;
    }

    for ($i = 0; $i < 10; $i++) {
        if ($labels[$i] === 0) {
            if (do_stuff($labels)) {
                return false;
            }
        }
    }

    return true;
}

/**
 * @param array<int, int> $labels
 */
function do_stuff(array &$labels): bool
{
    for ($i = 0; $i < 10; $i++) {
        $labels[$i] = 1;
    }
    return false;
}

function test_not_identical(): bool
{
    $values = [];

    for ($i = 0; $i < 5; $i++) {
        $values[$i] = 'initial';
    }

    for ($i = 0; $i < 5; $i++) {
        if ($values[$i] !== 'modified' && modify_values($values)) {
            return false;
        }
    }

    return true;
}

/**
 * @param array<int, string> $values
 */
function modify_values(array &$values): bool
{
    for ($i = 0; $i < 5; $i++) {
        $values[$i] = 'modified';
    }
    return false;
}
