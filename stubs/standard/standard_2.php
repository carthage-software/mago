<?php

use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Internal\ReturnTypeContract as TypeContract;
use JetBrains\PhpStorm\Pure;

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
 * Parses the string into variables
 * @link https://php.net/manual/en/function.parse-str.php
 * @param string $string <p>
 * The input string.
 * </p>
 * @param array &$result <p>
 * If the second parameter arr is present,
 * variables are stored in this variable as array elements instead.<br/>
 * Since 7.2.0 this parameter is not optional.
 * </p>
 * @return void
 */
function parse_str(
    string $string,
    #[PhpStormStubsElementAvailable(from: '5.3', to: '7.4')]  &$result = [],
    #[PhpStormStubsElementAvailable(from: '8.0')]  &$result,
): void {
}

/**
 * Parse a CSV string into an array
 * @link https://php.net/manual/en/function.str-getcsv.php
 * @param string $string <p>
 * The string to parse.
 * </p>
 * @param string $separator [optional] <p>
 * Set the field delimiter (one character only).
 * </p>
 * @param string $enclosure [optional] <p>
 * Set the field enclosure character (one character only).
 * </p>
 * @param string $escape [optional] <p>
 * Set the escape character (one character only).
 * Defaults as a backslash (\)
 * </p>
 * @return array an indexed array containing the fields read.
 *
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

/**
 * Parses input from a string according to a format
 * @link https://php.net/manual/en/function.sscanf.php
 * @param string $string <p>
 * The input string being parsed.
 * </p>
 * @param string $format <p>
 * The interpreted format for str, which is
 * described in the documentation for sprintf with
 * following differences:
 * Function is not locale-aware.
 * F, g, G and
 * b are not supported.
 * D stands for decimal number.
 * i stands for integer with base detection.
 * n stands for number of characters processed so far.
 * </p>
 * @param mixed &...$vars [optional]
 * @return array|int|null If only
 * two parameters were passed to this function, the values parsed
 * will be returned as an array. Otherwise, if optional parameters are passed,
 * the function will return the number of assigned values. The optional
 * parameters must be passed by reference.
 */
function sscanf(
    string $string,
    string $format,
    #[TypeContract(exists: 'int|null', notExists: 'array|null')] mixed &...$vars,
): array|int|null {
}

/**
 * Parses input from a file according to a format
 * @link https://php.net/manual/en/function.fscanf.php
 * @param resource $stream &fs.file.pointer;
 * @param string $format <p>
 * The specified format as described in the
 * sprintf documentation.
 * </p>
 * @param mixed &...$vars [optional]
 * @return array|int|false|null If only two parameters were passed to this function, the values parsed will be
 * returned as an array. Otherwise, if optional parameters are passed, the
 * function will return the number of assigned values. The optional
 * parameters must be passed by reference.
 */
function fscanf(
    $stream,
    string $format,
    #[TypeContract(exists: 'int|false|null', notExists: 'array|false|null')] mixed &...$vars,
): array|int|false|null {
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

function unlink(string $filename, $context): bool
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
 * Execute a command and open file pointers for input/output
 * @link https://php.net/manual/en/function.proc-open.php
 * @param array|string $command <p>
 * Execute a command and open file pointers for input/output
 * </p>
 * <p>
 * As of PHP 7.4.0, cmd may be passed as array of command parameters.
 * In this case the process will be opened directly
 * (without going through a shell) and PHP will take care of any
 * necessary argument escaping.
 * </p>
 * @param array $descriptor_spec <p>
 * An indexed array where the key represents the descriptor number and the
 * value represents how PHP will pass that descriptor to the child
 * process. 0 is stdin, 1 is stdout, while 2 is stderr.
 * </p>
 * <p>
 * Each element can be:
 * An array describing the pipe to pass to the process. The first
 * element is the descriptor type and the second element is an option for
 * the given type. Valid types are pipe (the second
 * element is either r to pass the read end of the pipe
 * to the process, or w to pass the write end) and
 * file (the second element is a filename).
 * A stream resource representing a real file descriptor (e.g. opened file,
 * a socket, STDIN).
 * </p>
 * <p>
 * The file descriptor numbers are not limited to 0, 1 and 2 - you may
 * specify any valid file descriptor number and it will be passed to the
 * child process. This allows your script to interoperate with other
 * scripts that run as "co-processes". In particular, this is useful for
 * passing passphrases to programs like PGP, GPG and openssl in a more
 * secure manner. It is also useful for reading status information
 * provided by those programs on auxiliary file descriptors.
 * </p>
 * @param array &$pipes <p>
 * Will be set to an indexed array of file pointers that correspond to
 * PHP's end of any pipes that are created.
 * </p>
 * @param string|null $cwd [optional] <p>
 * The initial working dir for the command. This must be an
 * absolute directory path, or null
 * if you want to use the default value (the working dir of the current
 * PHP process)
 * </p>
 * @param array|null $env_vars [optional] <p>
 * An array with the environment variables for the command that will be
 * run, or null to use the same environment as the current PHP process
 * </p>
 * @param array|null $options [optional] <p>
 * Allows you to specify additional options. Currently supported options
 * include:
 * suppress_errors (windows only): suppresses errors generated by this
 * function when it's set to TRUE
 * generated by this function when it's set to true
 * bypass_shell (windows only): bypass cmd.exe shell when set to TRUE
 * context: stream context used when opening files
 * (created with stream_context_create)
 * blocking_pipes: (windows only): force blocking pipes when set to TRUE
 * create_process_group (windows only): allow the child process to handle
 * CTRL events when set to TRUE
 * create_new_console (windows only): the new process has a new console,
 * instead of inheriting its parent's console
 * </p>
 * @return resource|false a resource representing the process, which should be freed using
 * proc_close when you are finished with it. On failure
 * returns false.
 */
function proc_open(
    array|string $command,
    array $descriptor_spec,
    &$pipes,
    null|string $cwd,
    null|array $env_vars,
    null|array $options,
) {
}

/**
 * Close a process opened by {@see proc_open} and return the exit code of that process
 * @link https://php.net/manual/en/function.proc-close.php
 * @param resource $process <p>
 * The proc_open resource that will
 * be closed.
 * </p>
 * @return int the termination status of the process that was run.
 */
function proc_close($process): int
{
}

/**
 * Kills a process opened by proc_open
 * @link https://php.net/manual/en/function.proc-terminate.php
 * @param resource $process <p>
 * The proc_open resource that will
 * be closed.
 * </p>
 * @param int $signal [optional] <p>
 * This optional parameter is only useful on POSIX
 * operating systems; you may specify a signal to send to the process
 * using the kill(2) system call. The default is
 * SIGTERM.
 * </p>
 * @return bool the termination status of the process that was run.
 */
function proc_terminate($process, int $signal = 15): bool
{
}

/**
 * Get information about a process opened by {@see proc_open}
 * @link https://php.net/manual/en/function.proc-get-status.php
 * @param resource $process <p>
 * The proc_open resource that will
 * be evaluated.
 * </p>
 * @return array|false An array of collected information on success, and false
 * on failure. The returned array contains the following elements:
 * </p>
 * <p>
 * <tr valign="top"><td>element</td><td>type</td><td>description</td></tr>
 * <tr valign="top">
 * <td>command</td>
 * <td>string</td>
 * <td>
 * The command string that was passed to proc_open.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>pid</td>
 * <td>int</td>
 * <td>process id</td>
 * </tr>
 * <tr valign="top">
 * <td>running</td>
 * <td>bool</td>
 * <td>
 * true if the process is still running, false if it has
 * terminated.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>signaled</td>
 * <td>bool</td>
 * <td>
 * true if the child process has been terminated by
 * an uncaught signal. Always set to false on Windows.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>stopped</td>
 * <td>bool</td>
 * <td>
 * true if the child process has been stopped by a
 * signal. Always set to false on Windows.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>exitcode</td>
 * <td>int</td>
 * <td>
 * The exit code returned by the process (which is only
 * meaningful if running is false).
 * Only first call of this function return real value, next calls return
 * -1.
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>termsig</td>
 * <td>int</td>
 * <td>
 * The number of the signal that caused the child process to terminate
 * its execution (only meaningful if signaled is true).
 * </td>
 * </tr>
 * <tr valign="top">
 * <td>stopsig</td>
 * <td>int</td>
 * <td>
 * The number of the signal that caused the child process to stop its
 * execution (only meaningful if stopped is true).
 * </td>
 * </tr>
 */
#[ArrayShape([
    'command' => 'string',
    'pid' => 'int',
    'running' => 'bool',
    'signaled' => 'bool',
    'stopped' => 'bool',
    'exitcode' => 'int',
    'termsig' => 'int',
    'stopsig' => 'int',
])]
#[LanguageLevelTypeAware(['8.0' => 'array'], default: 'array|false')]
function proc_get_status($process)
{
}

/**
 * Change the priority of the current process. <br/>
 * Since 7.2.0 supported on Windows platforms.
 * @link https://php.net/manual/en/function.proc-nice.php
 * @param int $priority <p>
 * The increment value of the priority change.
 * </p>
 * @return bool true on success or false on failure.
 * If an error occurs, like the user lacks permission to change the priority,
 * an error of level E_WARNING is also generated.
 */
function proc_nice(int $priority): bool
{
}

/**
 * Get port number associated with an Internet service and protocol
 * @link https://php.net/manual/en/function.getservbyname.php
 * @param string $service <p>
 * The Internet service name, as a string.
 * </p>
 * @param string $protocol <p>
 * protocol is either "tcp"
 * or "udp" (in lowercase).
 * </p>
 * @return int|false the port number, or false if service or
 * protocol is not found.
 */
#[Pure]
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
