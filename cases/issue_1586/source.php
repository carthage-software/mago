<?php

declare(strict_types=1);

/**
 * @param non-empty-list<non-empty-list<string>> $known
 */
function find_first_match(array $known, string $maker): int
{
    $fixed = 0;
    foreach ($known as $rxe) {
        foreach ($rxe as $rx) {
            if (!preg_match("/^{$rx}$/i", $maker)) {
                continue;
            }

            $fixed = 1;
            break;
        }

        if ($fixed) {
            break;
        }
    }

    return $fixed;
}

function find_make(string $maker): string
{
    $fixed = 0;
    $known = [
        'OM Systems' => ['OM Digital Solutions'],
        'Olympus' => ['Olympus.*'],
    ];
    foreach ($known as $name => $rxe) {
        foreach ($rxe as $rx) {
            if (!preg_match("/^{$rx}$/i", $maker)) {
                continue;
            }
            $fixed = 1;
            $maker = $name;
            break;
        }
        if ($fixed) {
            break;
        }
    }

    return $maker;
}
