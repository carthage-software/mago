<?php

declare(strict_types=1);

function test_isset_on_proc_open_pipes(): void
{
    $pipes = [];
    $process = @proc_open('echo hello', [0 => ['pipe', 'r'], 1 => ['pipe', 'w'], 2 => ['pipe', 'w']], $pipes);
    if (!is_resource($process)) {
        return;
    }

    if (isset($pipes[0])) {
        echo 'stdin';
    }

    if (isset($pipes[1])) {
        echo 'stdout';
    }

    if (isset($pipes[2])) {
        echo 'stderr';
    }
}
