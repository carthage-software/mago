<?php

function dl(string $extension_filename): bool
{
}

function cli_set_process_title(string $title): bool
{
}

/**
 * @pure
 */
function cli_get_process_title(): null|string
{
}

/**
 * @deprecated
 * @pure
 */
function utf8_encode(string $string): string
{
}

/**
 * @pure
 * @deprecated
 */
function utf8_decode(string $string): string
{
}

function error_clear_last(): void
{
}

function sapi_windows_cp_get(string $kind = ''): int
{
}

function sapi_windows_cp_set(int $codepage): bool
{
}

function sapi_windows_cp_conv(int|string $in_codepage, int|string $out_codepage, string $subject): null|string
{
}

function sapi_windows_cp_is_utf8(): bool
{
}

/**
 * @param resource $stream
 */
function sapi_windows_vt100_support($stream, null|bool $enable = null): bool
{
}

function sapi_windows_set_ctrl_handler(null|callable $handler, bool $add = true): bool
{
}

function sapi_windows_generate_ctrl_event(int $event, int $pid = 0): bool
{
}
