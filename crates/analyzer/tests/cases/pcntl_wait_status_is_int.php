<?php

declare(strict_types=1);

function reap(int $pid): int
{
    $status = 0;
    pcntl_waitpid($pid, $status);

    return $status;
}

/**
 * @return array<array-key, mixed>
 */
function wait_info(): array
{
    $info = [];
    pcntl_waitid(P_ALL, null, $info);

    return $info;
}
