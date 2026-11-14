<?php

declare(strict_types=1);

function break_type_0_pick(): int
{
    for ($i = 0; $i < 10; $i++) {
        if (rand(0, 1) === 1) {
            $foo = $i;
        }

        if (isset($foo)) {
            break;
        }
    }

    if (isset($foo) && $foo > 0) {
        return 1;
    }

    return 0;
}

function break_type_1_first(): int
{
    while (true) {
        if (rand(0, 1) === 1) {
            $foo = 1;
            break;
        }
    }

    return $foo;
}

/**
 * @param list<string> $list
 */
function break_type_1_found(array $list): bool
{
    $n = 0;
    foreach ($list as $value) {
        $n++;
        if ($value !== '') {
            $found = $n;
            break;
        }
    }

    return isset($found) && $found > 1;
}
