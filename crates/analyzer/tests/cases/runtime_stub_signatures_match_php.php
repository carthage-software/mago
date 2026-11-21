<?php

declare(strict_types=1);

function runtime_stub_output_buffers(): void
{
    ob_start('ob_gzhandler');
    ob_start(static fn(string $buffer, int $phase): string|false => $phase > 0 ? $buffer : false);
}

function runtime_stub_signal_handler_is_callable_or_int(): string
{
    $handler = pcntl_signal_get_handler(SIGUSR1);
    if (is_int($handler)) {
        return 'default or ignore';
    }

    return 'custom';
}

function runtime_stub_old_file(string $file): bool
{
    $modified = filemtime($file);

    return $modified !== false && $modified < 1_000_000_000;
}
