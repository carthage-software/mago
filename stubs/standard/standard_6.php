<?php

/**
 * @template R of null|array<array-key, resource>
 * @template W of null|array<array-key, resource>
 * @template E of null|array<array-key, resource>
 *
 * @param R $read
 * @param W $write
 * @param E $except
 *
 * @param-out (R is null ? null : array<array-key, resource>) $read
 * @param-out (W is null ? null : array<array-key, resource>) $write
 * @param-out (E is null ? null : array<array-key, resource>) $except
 *
 * @return false|int<0, max>
 */
function stream_select(
    null|array &$read,
    null|array &$write,
    null|array &$except,
    null|int $seconds,
    null|int $microseconds,
): int|false {
}

/**
 * @return resource
 */
function stream_context_create(null|array $options = null, null|array $params = null): mixed
{
}

/**
 * @param resource $context
 */
function stream_context_set_params($context, array $params): bool
{
}

/**
 * @param resource $context
 *
 * @return array{notification: string, options: array}
 */
function stream_context_get_params($context): array
{
}

/**
 * @param resource $context
 */
function stream_context_set_option($context, string $wrapper_or_options, string $option_name, mixed $value): bool
{
}

/**
 * @param resource $stream_or_context
 */
function stream_context_set_option($stream_or_context, array $options): bool
{
}

/**
 * @param resource $context
 */
function stream_context_set_options($context, array $options): bool
{
}

/**
 * @param resource $stream_or_context
 */
function stream_context_get_options($stream_or_context): array
{
}

/**
 * @return resource
 */
function stream_context_get_default(null|array $options)
{
}

/**
 * @return resource
 */
function stream_context_set_default(array $options)
{
}

/**
 * @param resource $stream
 *
 * @return resource
 */
function stream_filter_prepend($stream, string $filter_name, int $mode = 0, mixed $params = null)
{
}

/**
 * @param resource $stream
 *
 * @return resource|false
 */
function stream_filter_append($stream, string $filter_name, int $mode = 0, mixed $params = null)
{
}

/**
 * @param resource $stream_filter
 */
function stream_filter_remove($stream_filter): bool
{
}

/**
 * @param null|resource $context
 *
 * @param-out null|int $error_code
 * @param-out null|string $error_message
 *
 * @return resource|false
 */
function stream_socket_client(
    string $address,
    &$error_code = null,
    &$error_message = null,
    null|float $timeout = null,
    int $flags = STREAM_CLIENT_CONNECT,
    $context = null,
) {
}

/**
 * @param null|resource $context
 *
 * @param-out null|int $error_code
 * @param-out null|string $error_message
 *
 * @return resource|false
 */
function stream_socket_server(
    string $address,
    &$error_code = null,
    &$error_message = null,
    int $flags = STREAM_SERVER_BIND | STREAM_SERVER_LISTEN,
    $context = null,
) {
}

/**
 * @param resource $socket
 *
 * @param-out string $peer_name
 *
 * @return resource|false
 */
function stream_socket_accept($socket, null|float $timeout = null, &$peer_name = null)
{
}

/**
 * @param resource $socket
 */
function stream_socket_get_name($socket, bool $remote): string|false
{
}

/**
 * @param resource $socket
 *
 * @param-out string $address
 */
function stream_socket_recvfrom($socket, int $length, int $flags = 0, &$address): string|false
{
}

/**
 * @param resource $socket
 */
function stream_socket_sendto($socket, string $data, int $flags = 0, string $address = ''): int|false
{
}

/**
 * @param resource $stream
 * @param null|resource $session_stream
 */
function stream_socket_enable_crypto(
    $stream,
    bool $enable,
    null|int $crypto_method = null,
    $session_stream = null,
): int|bool {
}

/**
 * @param resource $stream
 */
function stream_socket_shutdown($stream, int $mode): bool
{
}

/**
 * @return list{resource, resource}|false
 */
function stream_socket_pair(int $domain, int $type, int $protocol): array|false
{
}

/**
 * @param resource $from
 * @param resource $to
 */
function stream_copy_to_stream($from, $to, null|int $length, int $offset = 0): int|false
{
}

/**
 * @param resource $stream
 */
function stream_get_contents($stream, null|int $length = null, int $offset = -1): string|false
{
}

/**
 * @param resource $stream
 */
function stream_supports_lock($stream): bool
{
}

/**
 * @param resource $stream
 */
function fgetcsv(
    $stream,
    null|int $length = null,
    string $separator = ',',
    string $enclosure = '"',
    string $escape = '\\',
): array|false {
}

/**
 * @param resource $stream
 */
function fputcsv(
    $stream,
    array $fields,
    string $separator = ',',
    string $enclosure = '"',
    string $escape = "\\",
    string $eol = PHP_EOL,
): int|false {
}

/**
 * @param resource $stream
 * @param-out int $would_block
 *
 * @return bool
 */
function flock($stream, int $operation, &$would_block): bool
{
}

/**
 * @return array<string, string>|false
 */
function get_meta_tags(string $filename, bool $use_include_path = false): array|false
{
}

/**
 * @param resource $stream
 */
function stream_set_write_buffer($stream, int $size): int
{
}

/**
 * @param resource $stream
 */
function stream_set_read_buffer($stream, int $size): int
{
}

/**
 * @param resource $stream
 */
function set_file_buffer($stream, int $size): int
{
}

/**
 * @param resource $stream
 */
function stream_set_blocking($stream, bool $enable): bool
{
}

/**
 * @param resource $stream
 */
function socket_set_blocking($stream, bool $enable): bool
{
}

/**
 * @param resource $stream
 *
 * @return array{
 *   'timed_out': bool,
 *   'blocked': bool,
 *   'eof': bool,
 *   'unread_bytes': int,
 *   'stream_type': string,
 *   'wrapper_type': string,
 *   'wrapper_data': mixed,
 *   'mode': string,
 *   'seekable': bool,
 *   'uri': string,
 *   'crypto': array,
 *   'mediatype': string,
 * }
 */
function stream_get_meta_data($stream): array
{
}

/**
 * @param resource $stream
 */
function stream_get_line($stream, int $length, string $ending = ''): string|false
{
}

function stream_wrapper_register(string $protocol, string $class, int $flags = 0): bool
{
}

function stream_register_wrapper(string $protocol, string $class, int $flags = 0): bool
{
}

function stream_resolve_include_path(string $filename): string|false
{
}

function stream_wrapper_unregister(string $protocol): bool
{
}

function stream_wrapper_restore(string $protocol): bool
{
}

/**
 * @return list<string>
 */
function stream_get_wrappers(): array
{
}

/**
 * @return list<string>
 */
function stream_get_transports(): array
{
}

/**
 * @param string|resource $stream
 */
function stream_is_local($stream): bool
{
}

/**
 * @param null|resource $context
 */
function get_headers(string $url, bool $associative = false, $context = null): array|false
{
}

/**
 * @param resource $stream
 */
function stream_set_timeout($stream, int $seconds, int $microseconds = 0): bool
{
}

/**
 * @param resource $stream
 */
function socket_set_timeout($stream, int $seconds, int $microseconds = 0): bool
{
}

/**
 * @param resource $stream
 */
function socket_get_status($stream): array
{
}

function realpath(string $path): string|false
{
}

function fnmatch(string $pattern, string $filename, int $flags = 0): bool
{
}
