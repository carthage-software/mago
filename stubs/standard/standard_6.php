<?php

use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Deprecated;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Pure;

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
 * Attach a filter to a stream
 * @link https://php.net/manual/en/function.stream-filter-append.php
 * @param resource $stream <p>
 * The target stream.
 * </p>
 * @param string $filter_name <p>
 * The filter name.
 * </p>
 * @param int $mode <p>
 * By default, stream_filter_append will
 * attach the filter to the read filter chain
 * if the file was opened for reading (i.e. File Mode:
 * r, and/or +). The filter
 * will also be attached to the write filter chain
 * if the file was opened for writing (i.e. File Mode:
 * w, a, and/or +).
 * STREAM_FILTER_READ,
 * STREAM_FILTER_WRITE, and/or
 * STREAM_FILTER_ALL can also be passed to the
 * read_write parameter to override this behavior.
 * </p>
 * @param mixed $params [optional] <p>
 * This filter will be added with the specified
 * params to the end of
 * the list and will therefore be called last during stream operations.
 * To add a filter to the beginning of the list, use
 * stream_filter_prepend.
 * </p>
 * @return resource|false a resource which can be used to refer to this filter
 * instance during a call to stream_filter_remove.
 */
function stream_filter_append($stream, string $filter_name, int $mode = 0, mixed $params)
{
}

/**
 * Remove a filter from a stream
 * @link https://php.net/manual/en/function.stream-filter-remove.php
 * @param resource $stream_filter <p>
 * The stream filter to be removed.
 * </p>
 * @return bool true on success or false on failure.
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
 * Retrieve the name of the local or remote sockets
 * @link https://php.net/manual/en/function.stream-socket-get-name.php
 * @param resource $socket <p>
 * The socket to get the name of.
 * </p>
 * @param bool $remote <p>
 * If set to true the remote socket name will be returned, if set
 * to false the local socket name will be returned.
 * </p>
 * @return string|false The name of the socket or false on error.
 */
function stream_socket_get_name($socket, bool $remote): string|false
{
}

/**
 * Receives data from a socket, connected or not
 * @link https://php.net/manual/en/function.stream-socket-recvfrom.php
 * @param resource $socket <p>
 * The remote socket.
 * </p>
 * @param int $length <p>
 * The number of bytes to receive from the socket.
 * </p>
 * @param int $flags <p>
 * The value of flags can be any combination
 * of the following:
 * <table>
 * Possible values for flags
 * <tr valign="top">
 * <td>STREAM_OOB</td>
 * <td>
 * Process OOB (out-of-band) data.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>STREAM_PEEK</td>
 * <td>
 * Retrieve data from the socket, but do not consume the buffer.
 * Subsequent calls to fread or
 * stream_socket_recvfrom will see
 * the same data.
 * </td>
 * </tr>
 * </table>
 * </p>
 * @param string &$address [optional] <p>
 * If address is provided it will be populated with
 * the address of the remote socket.
 * </p>
 * @return string|false the read data, as a string, or false on error
 */
function stream_socket_recvfrom($socket, int $length, int $flags = 0, &$address): string|false
{
}

/**
 * Sends a message to a socket, whether it is connected or not
 * @link https://php.net/manual/en/function.stream-socket-sendto.php
 * @param resource $socket <p>
 * The socket to send data to.
 * </p>
 * @param string $data <p>
 * The data to be sent.
 * </p>
 * @param int $flags <p>
 * The value of flags can be any combination
 * of the following:
 * <table>
 * possible values for flags
 * <tr valign="top">
 * <td>STREAM_OOB</td>
 * <td>
 * Process OOB (out-of-band) data.
 * </td>
 * </tr>
 * </table>
 * </p>
 * @param string $address <p>
 * The address specified when the socket stream was created will be used
 * unless an alternate address is specified in address.
 * </p>
 * <p>
 * If specified, it must be in dotted quad (or [ipv6]) format.
 * </p>
 * @return int|false a result code, as an integer.
 */
function stream_socket_sendto($socket, string $data, int $flags = 0, string $address = ''): int|false
{
}

/**
 * Turns encryption on/off on an already connected socket
 * @link https://php.net/manual/en/function.stream-socket-enable-crypto.php
 * @param resource $stream <p>
 * The stream resource.
 * </p>
 * @param bool $enable <p>
 * Enable/disable cryptography on the stream.
 * </p>
 * @param int|null $crypto_method [optional] <p>
 * Setup encryption on the stream.
 * Valid methods are:<br>
 * STREAM_CRYPTO_METHOD_SSLv2_CLIENT</p>
 * @param resource $session_stream [optional] <p>
 * Seed the stream with settings from session_stream.
 * </p>
 * @return bool|int true on success, false if negotiation has failed or
 * 0 if there isn't enough data and you should try again
 * (only for non-blocking sockets).
 */
function stream_socket_enable_crypto($stream, bool $enable, null|int $crypto_method, $session_stream): int|bool
{
}

/**
 * Shutdown a full-duplex connection
 * @link https://php.net/manual/en/function.stream-socket-shutdown.php
 * @param resource $stream <p>
 * An open stream (opened with stream_socket_client,
 * for example)
 * </p>
 * @param int $mode <p>
 * One of the following constants: STREAM_SHUT_RD
 * (disable further receptions), STREAM_SHUT_WR
 * (disable further transmissions) or
 * STREAM_SHUT_RDWR (disable further receptions and
 * transmissions).
 * </p>
 * @return bool true on success or false on failure.
 * @since 5.2.1
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
 * Copies data from one stream to another
 * @link https://php.net/manual/en/function.stream-copy-to-stream.php
 * @param resource $from <p>
 * The source stream
 * </p>
 * @param resource $to <p>
 * The destination stream
 * </p>
 * @param int|null $length [optional] <p>
 * Maximum bytes to copy
 * </p>
 * @param int $offset <p>
 * The offset where to start to copy data
 * </p>
 * @return int|false the total count of bytes copied, or false on failure.
 */
function stream_copy_to_stream($from, $to, null|int $length, int $offset = 0): int|false
{
}

/**
 * Reads remainder of a stream into a string
 * @link https://php.net/manual/en/function.stream-get-contents.php
 * @param resource $stream <p>
 * A stream resource (e.g. returned from fopen)
 * </p>
 * @param int|null $length <p>
 * The maximum bytes to read. Defaults to -1 (read all the remaining
 * buffer).
 * </p>
 * @param int $offset [optional] <p>
 * Seek to the specified offset before reading.
 * </p>
 * @return string|false a string or false on failure.
 */
function stream_get_contents($stream, null|int $length = null, int $offset = -1): string|false
{
}

/**
 * Tells whether the stream supports locking.
 * @link https://php.net/manual/en/function.stream-supports-lock.php
 * @param resource $stream <p>
 * The stream to check.
 * </p>
 * @return bool true on success or false on failure.
 */
function stream_supports_lock($stream): bool
{
}

/**
 * Gets line from file pointer and parse for CSV fields
 * @link https://php.net/manual/en/function.fgetcsv.php
 * @param resource $stream <p>
 * A valid file pointer to a file successfully opened by
 * fopen, popen, or
 * fsockopen.
 * </p>
 * @param int|null $length <p>
 * Must be greater than the longest line (in characters) to be found in
 * the CSV file (allowing for trailing line-end characters). It became
 * optional in PHP 5. Omitting this parameter (or setting it to 0 in PHP
 * 5.0.4 and later) the maximum line length is not limited, which is
 * slightly slower.
 * </p>
 * @param string $separator [optional] <p>
 * Set the field delimiter (one character only).
 * </p>
 * @param string $enclosure [optional] <p>
 * Set the field enclosure character (one character only).
 * </p>
 * @param string $escape [optional] <p>
 * Set the escape character (one character only). Defaults as a backslash.
 * </p>
 * @return array|false|null an indexed array containing the fields read.
 * <p>
 * A blank line in a CSV file will be returned as an array
 * comprising a single null field, and will not be treated
 * as an error.
 * </p>
 * <p>
 * fgetcsv returns null if an invalid
 * handle is supplied or false on other errors,
 * including end of file.
 * </p>
 */
#[LanguageLevelTypeAware(['8.0' => 'array|false'], default: 'array|false|null')]
function fgetcsv(
    $stream,
    null|int $length = null,
    string $separator = ',',
    string $enclosure = '"',
    string $escape = '\\',
) {
}

/**
 * Format line as CSV and write to file pointer
 * @link https://php.net/manual/en/function.fputcsv.php
 * @param resource $stream The file pointer must be valid, and must point to a file successfully opened by fopen() or fsockopen() (and not yet closed by fclose()).
 * @param array $fields <p>
 * An array of values.
 * </p>
 * @param string $separator [optional] <p>
 * The optional delimiter parameter sets the field
 * delimiter (one character only).
 * </p>
 * @param string $enclosure [optional] <p>
 * The optional enclosure parameter sets the field
 * enclosure (one character only).
 * </p>
 * @param string $escape [optional] <p>
 * The optional escape_char parameter sets the escape character (one character only).
 * </p>
 * @return int|false the length of the written string or false on failure.
 */
function fputcsv(
    $stream,
    array $fields,
    string $separator = ',',
    string $enclosure = '"',
    #[PhpStormStubsElementAvailable(from: '7.0')] string $escape = "\\",
    #[PhpStormStubsElementAvailable('8.1')] string $eol = PHP_EOL,
): int|false {
}

/**
 * Portable advisory file locking
 * @link https://php.net/manual/en/function.flock.php
 * @param resource $stream <p>
 * An open file pointer.
 * </p>
 * @param int $operation <p>
 * operation is one of the following:
 * LOCK_SH to acquire a shared lock (reader).</p>
 * @param int &$would_block [optional] <p>
 * The optional third argument is set to 1 if the lock would block
 * (EWOULDBLOCK errno condition).
 * </p>
 * @return bool true on success or false on failure.
 */
function flock($stream, int $operation, &$would_block): bool
{
}

/**
 * Extracts all meta tag content attributes from a file and returns an array
 * @link https://php.net/manual/en/function.get-meta-tags.php
 * @param string $filename <p>
 * The path to the HTML file, as a string. This can be a local file or an
 * URL.
 * </p>
 * <p>
 * What get_meta_tags parses
 * </p>
 * <pre>
 * <meta name="author" content="name">
 * <meta name="keywords" content="php documentation">
 * <meta name="DESCRIPTION" content="a php manual">
 * <meta name="geo.position" content="49.33;-86.59">
 * </head> <!-- parsing stops here -->
 * </pre>
 * <p>
 * (pay attention to line endings - PHP uses a native function to
 * parse the input, so a Mac file won't work on Unix).
 * </p>
 * @param bool $use_include_path [optional] <p>
 * Setting use_include_path to true will result
 * in PHP trying to open the file along the standard include path as per
 * the include_path directive.
 * This is used for local files, not URLs.
 * </p>
 * @return array|false an array with all the parsed meta tags.
 * <p>
 * The value of the name property becomes the key, the value of the content
 * property becomes the value of the returned array, so you can easily use
 * standard array functions to traverse it or access single values.
 * Special characters in the value of the name property are substituted with
 * '_', the rest is converted to lower case. If two meta tags have the same
 * name, only the last one is returned.
 * </p>
 */
#[Pure(true)]
function get_meta_tags(string $filename, bool $use_include_path = false): array|false
{
}

/**
 * Sets write file buffering on the given stream
 * @link https://php.net/manual/en/function.stream-set-write-buffer.php
 * @param resource $stream <p>
 * The file pointer.
 * </p>
 * @param int $size <p>
 * The number of bytes to buffer. If buffer
 * is 0 then write operations are unbuffered. This ensures that all writes
 * with fwrite are completed before other processes are
 * allowed to write to that output stream.
 * </p>
 * @return int 0 on success, or EOF if the request cannot be honored.
 * @see stream_set_read_buffer()
 */
function stream_set_write_buffer($stream, int $size): int
{
}

/**
 * Sets read file buffering on the given stream
 * @link https://php.net/manual/en/function.stream-set-read-buffer.php
 * @param resource $stream <p>
 * The file pointer.
 * </p>
 * @param int $size <p>
 * The number of bytes to buffer. If buffer
 * is 0 then write operations are unbuffered. This ensures that all writes
 * with fwrite are completed before other processes are
 * allowed to write to that output stream.
 * </p>
 * @return int 0 on success, or EOF if the request cannot be honored.
 * @see stream_set_write_buffer()
 */
function stream_set_read_buffer($stream, int $size): int
{
}

/**
 * Alias:
 * {@see stream_set_write_buffer}
 * <p>Sets the buffering for write operations on the given stream to buffer bytes.
 * Output using fwrite() is normally buffered at 8K.
 * This means that if there are two processes wanting to write to the same output stream (a file),
 * each is paused after 8K of data to allow the other to write.
 * </p>
 * @link https://php.net/manual/en/function.set-file-buffer.php
 * @param resource $stream The file pointer.
 * @param int $size The number of bytes to buffer. If buffer is 0 then write operations are unbuffered.
 * This ensures that all writes with fwrite() are completed before other processes are allowed to write to that output stream.
 * @return int
 */
function set_file_buffer($stream, int $size): int
{
}

/**
 * Set blocking/non-blocking mode on a stream
 * @link https://php.net/manual/en/function.stream-set-blocking.php
 * @param resource $stream <p>
 * The stream.
 * </p>
 * @param bool $enable <p>
 * If mode is FALSE, the given stream
 * will be switched to non-blocking mode, and if TRUE, it
 * will be switched to blocking mode. This affects calls like
 * fgets and fread
 * that read from the stream. In non-blocking mode an
 * fgets call will always return right away
 * while in blocking mode it will wait for data to become available
 * on the stream.
 * </p>
 * @return bool true on success or false on failure.
 */
function stream_set_blocking($stream, bool $enable): bool
{
}

/**
 * Alias:
 * {@see stream_set_blocking}
 * @link https://php.net/manual/en/function.socket-set-blocking.php
 * @param resource $stream <p>
 * The stream.
 * </p>
 * @param bool $enable <p>
 * If mode is FALSE, the given stream
 * will be switched to non-blocking mode, and if TRUE, it
 * will be switched to blocking mode. This affects calls like
 * fgets and fread
 * that read from the stream. In non-blocking mode an
 * fgets call will always return right away
 * while in blocking mode it will wait for data to become available
 * on the stream.
 * </p>
 * @return bool true on success or false on failure.
 */
function socket_set_blocking($stream, bool $enable): bool
{
}

/**
 * Retrieves header/meta data from streams/file pointers
 * @link https://php.net/manual/en/function.stream-get-meta-data.php
 * @param resource $stream <p>
 * The stream can be any stream created by fopen,
 * fsockopen and pfsockopen.
 * </p>
 * @return array The result array contains the following items:
 * <p>
 * timed_out (bool) - true if the stream
 * timed out while waiting for data on the last call to
 * fread or fgets.
 * </p>
 * <p>
 * blocked (bool) - true if the stream is
 * in blocking IO mode. See stream_set_blocking.
 * </p>
 * <p>
 * eof (bool) - true if the stream has reached
 * end-of-file. Note that for socket streams this member can be true
 * even when unread_bytes is non-zero. To
 * determine if there is more data to be read, use
 * feof instead of reading this item.
 * </p>
 * <p>
 * unread_bytes (int) - the number of bytes
 * currently contained in the PHP's own internal buffer.
 * </p>
 * You shouldn't use this value in a script.
 * <p>
 * stream_type (string) - a label describing
 * the underlying implementation of the stream.
 * </p>
 * <p>
 * wrapper_type (string) - a label describing
 * the protocol wrapper implementation layered over the stream.
 * See for more information about wrappers.
 * </p>
 * <p>
 * wrapper_data (mixed) - wrapper specific
 * data attached to this stream. See for
 * more information about wrappers and their wrapper data.
 * </p>
 * <p>
 * filters (array) - and array containing
 * the names of any filters that have been stacked onto this stream.
 * Documentation on filters can be found in the
 * Filters appendix.
 * </p>
 * <p>
 * mode (string) - the type of access required for
 * this stream (see Table 1 of the fopen() reference)
 * </p>
 * <p>
 * seekable (bool) - whether the current stream can
 * be seeked.
 * </p>
 * <p>
 * uri (string) - the URI/filename associated with this
 * stream.
 * </p>
 */
#[ArrayShape([
    'timed_out' => 'bool',
    'blocked' => 'bool',
    'eof' => 'bool',
    'unread_bytes' => 'int',
    'stream_type' => 'string',
    'wrapper_type' => 'string',
    'wrapper_data' => 'mixed',
    'mode' => 'string',
    'seekable' => 'bool',
    'uri' => 'string',
    'crypto' => 'array',
    'mediatype' => 'string',
])]
function stream_get_meta_data($stream): array
{
}

/**
 * Gets line from stream resource up to a given delimiter
 * @link https://php.net/manual/en/function.stream-get-line.php
 * @param resource $stream <p>
 * A valid file handle.
 * </p>
 * @param int $length <p>
 * The number of bytes to read from the handle.
 * </p>
 * @param string $ending <p>
 * An optional string delimiter.
 * </p>
 * @return string|false a string of up to length bytes read from the file
 * pointed to by handle.
 * <p>
 * If an error occurs, returns false.
 * </p>
 */
function stream_get_line($stream, int $length, string $ending = ''): string|false
{
}

/**
 * Register a URL wrapper implemented as a PHP class
 * @link https://php.net/manual/en/function.stream-wrapper-register.php
 * @param string $protocol <p>
 * The wrapper name to be registered.
 * </p>
 * @param string $class <p>
 * The classname which implements the protocol.
 * </p>
 * @param int $flags <p>
 * Should be set to STREAM_IS_URL if
 * protocol is a URL protocol. Default is 0, local
 * stream.
 * </p>
 * @return bool true on success or false on failure.
 * <p>
 * stream_wrapper_register will return false if the
 * protocol already has a handler.
 * </p>
 */
function stream_wrapper_register(string $protocol, string $class, int $flags = 0): bool
{
}

/**
 * Alias:
 * {@see stream_wrapper_register}
 * <p>Register a URL wrapper implemented as a PHP class</p>
 * @link https://php.net/manual/en/function.stream-register-wrapper.php
 * @param string $protocol <p>
 * The wrapper name to be registered.
 * </p>
 * @param string $class <p>
 * The classname which implements the protocol.
 * </p>
 * @param int $flags [optional] <p>
 * Should be set to STREAM_IS_URL if
 * protocol is a URL protocol. Default is 0, local
 * stream.
 * </p>
 * @return bool true on success or false on failure.
 * <p>
 * stream_wrapper_register will return false if the
 * protocol already has a handler.
 * </p>
 */
function stream_register_wrapper(string $protocol, string $class, int $flags = 0): bool
{
}

/**
 * Resolve filename against the include path according to the same rules as fopen()/include().
 * @link https://php.net/manual/en/function.stream-resolve-include-path.php
 * @param string $filename The filename to resolve.
 * @return string|false containing the resolved absolute filename, or FALSE on failure.
 * @since 5.3.2
 */
function stream_resolve_include_path(string $filename): string|false
{
}

/**
 * Unregister a URL wrapper
 * @link https://php.net/manual/en/function.stream-wrapper-unregister.php
 * @param string $protocol <p>
 * </p>
 * @return bool true on success or false on failure.
 */
function stream_wrapper_unregister(string $protocol): bool
{
}

/**
 * Restores a previously unregistered built-in wrapper
 * @link https://php.net/manual/en/function.stream-wrapper-restore.php
 * @param string $protocol <p>
 * </p>
 * @return bool true on success or false on failure.
 */
function stream_wrapper_restore(string $protocol): bool
{
}

/**
 * Retrieve list of registered streams
 * @link https://php.net/manual/en/function.stream-get-wrappers.php
 * @return list<string> an indexed array containing the name of all stream wrappers
 * available on the running system.
 */
#[Pure(true)]
function stream_get_wrappers(): array
{
}

/**
 * Retrieve list of registered socket transports
 * @link https://php.net/manual/en/function.stream-get-transports.php
 * @return list<string> an indexed array of socket transports names.
 */
#[Pure(true)]
function stream_get_transports(): array
{
}

/**
 * Checks if a stream is a local stream
 * @link https://php.net/manual/en/function.stream-is-local.php
 * @param mixed $stream <p>
 * The stream resource or URL to check.
 * </p>
 * @return bool true on success or false on failure.
 * @since 5.2.4
 */
#[Pure]
function stream_is_local($stream): bool
{
}

/**
 * Fetches all the headers sent by the server in response to an HTTP request
 * @link https://php.net/manual/en/function.get-headers.php
 * @param string $url <p>
 * The target URL.
 * </p>
 * @param bool $associative [optional] <p>
 * If the optional format parameter is set to true,
 * get_headers parses the response and sets the
 * array's keys.
 * </p>
 * @param resource $context [optional]
 * @return array|false an indexed or associative array with the headers, or false on
 * failure.
 */
#[Pure(true)]
function get_headers(
    string $url,
    #[LanguageLevelTypeAware(['8.0' => 'bool'], default: 'int')]  $associative = false,
    #[PhpStormStubsElementAvailable(from: '7.1')]  $context = null,
): array|false {
}

/**
 * Set timeout period on a stream
 * @link https://php.net/manual/en/function.stream-set-timeout.php
 * @param resource $stream <p>
 * The target stream.
 * </p>
 * @param int $seconds <p>
 * The seconds part of the timeout to be set.
 * </p>
 * @param int $microseconds <p>
 * The microseconds part of the timeout to be set.
 * </p>
 * @return bool true on success or false on failure.
 */
function stream_set_timeout(
    $stream,
    int $seconds,
    #[PhpStormStubsElementAvailable(from: '5.3', to: '5.6')] int $microseconds,
    #[PhpStormStubsElementAvailable(from: '7.0')] int $microseconds = 0,
): bool {
}

/**
 * Alias:
 * {@see stream_set_timeout}
 * Set timeout period on a stream
 * @link https://php.net/manual/en/function.socket-set-timeout.php
 * @param resource $stream <p>
 * The target stream.
 * </p>
 * @param int $seconds <p>
 * The seconds part of the timeout to be set.
 * </p>
 * @param int $microseconds <p>
 * The microseconds part of the timeout to be set.
 * </p>
 * @return bool true on success or false on failure.
 */
function socket_set_timeout(
    $stream,
    int $seconds,
    #[PhpStormStubsElementAvailable(from: '5.3', to: '5.6')] int $microseconds,
    #[PhpStormStubsElementAvailable(from: '7.0')] int $microseconds = 0,
): bool {
}

/**
 * Alias:
 * {@see stream_get_meta_data}
 * Retrieves header/meta data from streams/file pointers
 * @link https://php.net/manual/en/function.socket-get-status.php
 * @param resource $stream <p>
 * The stream can be any stream created by fopen,
 * fsockopen and pfsockopen.
 * </p>
 * @return array The result array contains the following items:
 * <p>
 * timed_out (bool) - true if the stream
 * timed out while waiting for data on the last call to
 * fread or fgets.
 * </p>
 * <p>
 * blocked (bool) - true if the stream is
 * in blocking IO mode. See stream_set_blocking.
 * </p>
 * <p>
 * eof (bool) - true if the stream has reached
 * end-of-file. Note that for socket streams this member can be true
 * even when unread_bytes is non-zero. To
 * determine if there is more data to be read, use
 * feof instead of reading this item.
 * </p>
 * <p>
 * unread_bytes (int) - the number of bytes
 * currently contained in the PHP's own internal buffer.
 * </p>
 * You shouldn't use this value in a script.
 * <p>
 * stream_type (string) - a label describing
 * the underlying implementation of the stream.
 * </p>
 * <p>
 * wrapper_type (string) - a label describing
 * the protocol wrapper implementation layered over the stream.
 * See for more information about wrappers.
 * </p>
 * <p>
 * wrapper_data (mixed) - wrapper specific
 * data attached to this stream. See for
 * more information about wrappers and their wrapper data.
 * </p>
 * <p>
 * filters (array) - and array containing
 * the names of any filters that have been stacked onto this stream.
 * Documentation on filters can be found in the
 * Filters appendix.
 * </p>
 * <p>
 * mode (string) - the type of access required for
 * this stream (see Table 1 of the fopen() reference)
 * </p>
 * <p>
 * seekable (bool) - whether the current stream can
 * be seeked.
 * </p>
 * <p>
 * uri (string) - the URI/filename associated with this
 * stream.
 * </p>
 */
function socket_get_status($stream): array
{
}

/**
 * Returns canonicalized absolute pathname
 * @link https://php.net/manual/en/function.realpath.php
 * @param string $path <p>
 * The path being checked.
 * </p>
 * @return string|false the canonicalized absolute pathname on success. The resulting path
 * will have no symbolic link, '/./' or '/../' components.
 * <p>
 * realpath returns false on failure, e.g. if
 * the file does not exist.
 * </p>
 */
#[Pure(true)]
function realpath(string $path): string|false
{
}

/**
 * Match filename against a pattern
 * @link https://php.net/manual/en/function.fnmatch.php
 * @param string $pattern <p>
 * The shell wildcard pattern.
 * </p>
 * @param string $filename <p>
 * The tested string. This function is especially useful for filenames,
 * but may also be used on regular strings.
 * </p>
 * <p>
 * The average user may be used to shell patterns or at least in their
 * simplest form to '?' and '*'
 * wildcards so using fnmatch instead of
 * preg_match for
 * frontend search expression input may be way more convenient for
 * non-programming users.
 * </p>
 * @param int $flags <p>
 * The value of flags can be any combination of
 * the following flags, joined with the
 * binary OR (|) operator.
 * <table>
 * A list of possible flags for fnmatch
 * <tr valign="top">
 * <td>Flag</td>
 * <td>Description</td>
 * </tr>
 * <tr valign="top">
 * <td>FNM_NOESCAPE</td>
 * <td>
 * Disable backslash escaping.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>FNM_PATHNAME</td>
 * <td>
 * Slash in string only matches slash in the given pattern.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>FNM_PERIOD</td>
 * <td>
 * Leading period in string must be exactly matched by period in the given pattern.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>FNM_CASEFOLD</td>
 * <td>
 * Caseless match. Part of the GNU extension.
 * </td>
 * </tr>
 * </table>
 * </p>
 * @return bool true if there is a match, false otherwise.
 */
#[Pure(true)]
function fnmatch(string $pattern, string $filename, int $flags = 0): bool
{
}
