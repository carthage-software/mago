<?php

declare(strict_types=1);

/** @return resource|false */
function open_with_environment(string $command)
{
    $pipes = [];

    return proc_open($command, [1 => ['pipe', 'w']], $pipes, null, $_ENV);
}

/** @return resource|false */
function open_with_filtered_environment(string $command)
{
    $pipes = [];
    $environment = array_filter($_ENV, static fn(string $value): bool => $value !== '');

    return proc_open($command, [1 => ['pipe', 'w']], $pipes, null, $environment);
}

function debug_flag(): bool
{
    return ($_ENV['APP_DEBUG'] ?? '0') === '1';
}
