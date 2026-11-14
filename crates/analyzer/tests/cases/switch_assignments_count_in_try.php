<?php


declare(strict_types=1);

function catch_leaves(string $s): int
{
    try {
        switch ($s) {
            case 'S':
                $x = 1;
                break;
            default:
                $x = 3;
                break;
        }
    } catch (Throwable) {
        return 0;
    }

    return $x;
}

function only_finally(string $s): int
{
    try {
        switch ($s) {
            case 'S':
                $x = 1;
                break;
            default:
                $x = 3;
                break;
        }
    } finally {
        echo 'done';
    }

    return $x;
}
