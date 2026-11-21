<?php

declare(strict_types=1);

function switch_true_fallthrough_case_sees_earlier_cases(object|string $connection): int
{
    switch (true) {
        case !is_string($connection):
            return 1;
        case str_starts_with($connection, 'file://'):
            return 2;
        case str_starts_with($connection, 'redis:'):
        case str_starts_with($connection, 'rediss:'):
            return 3;
    }

    return 0;
}
