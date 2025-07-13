<?php

/**
 * @pure
 */
function nl_langinfo(int $item): string|false
{
}

/**
 * @pure
 */
function soundex(string $string): string
{
}

function levenshtein(
    string $string1,
    string $string2,
    int $insertion_cost = 1,
    int $replacement_cost = 1,
    int $deletion_cost = 1,
): int {
}

/**
 * @pure
 */
function chr(int $codepoint): string
{
}

/**
 * @param string $character
 * @return int<0, 255>
 *
 * @pure
 */
function ord(string $character): int
{
}

/**
 * @param-out array<string, string> $result
 *
 * @return void
 */
function parse_str(string $string, &$result): void
{
}

/**
 * @pure
 */
function str_getcsv(string $string, string $separator = ',', string $enclosure = '"', string $escape = "\\"): array
{
}

/**
 * @pure
 */
function str_pad(string $string, int $length, string $pad_string = ' ', int $pad_type = STR_PAD_RIGHT): string
{
}

/**
 * @pure
 */
function chop(string $string, string $characters = " \n\r\t\v\0"): string
{
}

/**
 * @pure
 */
function strchr(string $haystack, string $needle, bool $before_needle = false): string|false
{
}

/**
 * @param string|int|float ...$values
 *
 * @pure
 */
function sprintf(string $format, mixed ...$values): string
{
}

/**
 * @param string|int|float ...$values
 *
 * @return int<0, max>
 */
function printf(string $format, mixed ...$values): int
{
}

/**
 * @pure
 */
function vprintf(string $format, array $values): int
{
}

/**
 * @pure
 */
function vsprintf(string $format, array $values): string
{
}

/**
 * @param resource $stream
 *
 * @pure
 */
function fprintf($stream, string $format, mixed ...$values): int
{
}

/**
 * @param resource $stream
 *
 * @pure
 */
function vfprintf($stream, string $format, array $values): int
{
}

function sscanf(string $string, string $format, mixed &...$vars): array|int|null
{
}

/**
 * @param resource $stream
 */
function fscanf($stream, string $format, mixed &...$vars): array|int|false|null
{
}

/**
 * @pure
 */
function parse_url(string $url, int $component = -1): array|string|int|false|null
{
}

/**
 * @pure
 */
function urlencode(string $string): string
{
}

/**
 * @pure
 */
function urldecode(string $string): string
{
}

/**
 * @pure
 */
function rawurlencode(string $string): string
{
}

/**
 * @pure
 */
function rawurldecode(string $string): string
{
}

/**
 * @pure
 */
function http_build_query(
    object|array $data,
    string $numeric_prefix = '',
    null|string $arg_separator = null,
    int $encoding_type = PHP_QUERY_RFC1738,
): string {
}

function readlink(string $path): string|false
{
}

function linkinfo(string $path): int|false
{
}

function symlink(string $target, string $link): bool
{
}

function link(string $target, string $link): bool
{
}

/**
 * @param null|resource $context
 */
function unlink(string $filename, mixed $context = null): bool
{
}

function exec(string $command, &$output, &$result_code): string|false
{
}

function system(string $command, &$result_code): string|false
{
}

/**
 * @pure
 */
function escapeshellcmd(string $command): string
{
}

/**
 * @pure
 */
function escapeshellarg(string $arg): string
{
}

/**
 * @pure
 */
function passthru(string $command, &$result_code): null|false
{
}

function shell_exec(string $command): string|false|null
{
}

/**
 * @param array<string>|string $command
 * @param array{
 *   0?: resource|array{0: 'pipe', 1: 'r'|'w'}|array{0: 'file', 1: non-empty-string},
 *   1?: resource|array{0: 'pipe', 1: 'r'|'w'}|array{0: 'file', 1: non-empty-string},
 *   2?: resource|array{0: 'pipe', 1: 'r'|'w'}|array{0: 'file', 1: non-empty-string},
 * } $descriptor_spec
 * @param non-empty-string|null $cwd
 * @param null|array<string, string> $env_vars
 * @param null|array{
 *   suppress_errors?: bool,
 *   bypass_shell?: bool,
 *   blocking_pipes?: bool,
 *   create_process_group?: bool,
 *   create_new_console?: bool,
 * } $options
 *
 * @param-out array{
 *   0: resource,
 *   1: resource,
 *   2: resource,
 * } $pipes
 *
 * @return open-resource|false
 */
function proc_open(
    array|string $command,
    array $descriptor_spec,
    null|array &$pipes,
    null|string $cwd = null,
    null|array $env_vars = null,
    null|array $options = null,
) {
}

/**
 * @param resource $process
 */
function proc_close($process): int
{
}

/**
 * @param resource $process
 */
function proc_terminate($process, int $signal = 15): bool
{
}

/**
 * @param resource $process
 *
 * @return array{
 *  'command': string,
 *  'pid': int,
 *  'running': bool,
 *  'signaled': bool,
 *  'stopped': bool,
 *  'exitcode': int,
 *  'termsig': int,
 *  'stopsig': int,
 * }
 */
function proc_get_status($process): array
{
}

function proc_nice(int $priority): bool
{
}

function getservbyname(string $service, string $protocol): int|false
{
}

/**
 * @pure
 */
function getservbyport(int $port, string $protocol): string|false
{
}

/**
 * @pure
 */
function getprotobyname(string $protocol): int|false
{
}

/**
 * @pure
 */
function getprotobynumber(int $protocol): string|false
{
}

/**
 * @pure
 */
function getmyuid(): int|false
{
}

/**
 * @pure
 */
function getmygid(): int|false
{
}

/**
 * @pure
 */
function getmypid(): int|false
{
}

/**
 * @pure
 */
function getmyinode(): int|false
{
}
