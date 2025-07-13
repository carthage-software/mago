<?php

use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Deprecated;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Pure;

/**
 * Open Internet or Unix domain socket connection
 * @link https://php.net/manual/en/function.fsockopen.php
 * @param string $hostname <p>
 * If you have compiled in OpenSSL support, you may prefix the
 * hostname with either ssl://
 * or tls:// to use an SSL or TLS client connection
 * over TCP/IP to connect to the remote host.
 * </p>
 * @param int $port <p>
 * The port number.
 * </p>
 * @param int &$error_code [optional] <p>
 * If provided, holds the system level error number that occurred in the
 * system-level connect() call.
 * </p>
 * <p>
 * If the value returned in errno is
 * 0 and the function returned false, it is an
 * indication that the error occurred before the
 * connect() call. This is most likely due to a
 * problem initializing the socket.
 * </p>
 * @param string &$error_message [optional] <p>
 * The error message as a string.
 * </p>
 * @param float|null $timeout [optional] <p>
 * The connection timeout, in seconds.
 * </p>
 * <p>
 * If you need to set a timeout for reading/writing data over the
 * socket, use stream_set_timeout, as the
 * timeout parameter to
 * fsockopen only applies while connecting the
 * socket.
 * </p>
 * @return resource|false fsockopen returns a file pointer which may be used
 * together with the other file functions (such as
 * fgets, fgetss,
 * fwrite, fclose, and
 * feof). If the call fails, it will return false
 */
function fsockopen(string $hostname, int $port = -1, &$error_code, &$error_message, null|float $timeout)
{
}

/**
 * Open persistent Internet or Unix domain socket connection
 * @link https://php.net/manual/en/function.pfsockopen.php
 * @see fsockopen
 * @param string $hostname
 * @param int $port
 * @param int &$error_code [optional]
 * @param string &$error_message [optional]
 * @param float|null $timeout [optional]
 * @return resource|false
 */
function pfsockopen(string $hostname, int $port = -1, &$error_code, &$error_message, null|float $timeout)
{
}

/**
 * @pure
 */
function pack(string $format, mixed ...$values): string
{
}

/**
 * @return ($format is 'a'|'A'|'h'|'H' ? array{1: string}|false : (
 *   $format is 'c' ? array{1: int<-128, 127>}|false : (
 *     $format is 'C' ? array{1: int<0, 255>}|false : (
 *       $format is 's' ? array{1: int<-32768, 32767>}|false : (
 *         $format is 'S'|'n'|'v' ? array{1: int<0, 65535>}|false : (
 *           $format is 'l' ? array{1: int<-2147483648, 2147483647>}|false : (
 *             $format is 'L'|'N'|'V' ? array{1: int<0, 4294967295>}|false : (
 *               $format is 'q'|'Q'|'J'|'P' ? array{1: int}|false : (
 *                 $format is 'f'|'g'|'G'|'d'|'e'|'E' ? array{1: float}|false : (
 *                   array<int>|false
 *                 )
 *               )
 *             )
 *           )
 *         )
 *       )
 *     )
 *   )
 * ))
 *
 * @pure
 */
function unpack(string $format, string $string, int $offset = 0): array|false
{
}

/**
 * Tells what the user's browser is capable of
 * @link https://php.net/manual/en/function.get-browser.php
 * @param string|null $user_agent [optional] <p>
 * The User Agent to be analyzed. By default, the value of HTTP
 * User-Agent header is used; however, you can alter this (i.e., look up
 * another browser's info) by passing this parameter.
 * </p>
 * <p>
 * You can bypass this parameter with a null value.
 * </p>
 * @param bool $return_array [optional] <p>
 * If set to true, this function will return an array
 * instead of an object.
 * </p>
 * @return array|object|false Returns false if browscap.ini can't be loaded or the user agent can't be found, otherwise the information is returned in an object or an array which will contain
 * various data elements representing, for instance, the browser's major and
 * minor version numbers and ID string; true/false values for features
 * such as frames, JavaScript, and cookies; and so forth.
 * </p>
 * <p>
 * The cookies value simply means that the browser
 * itself is capable of accepting cookies and does not mean the user has
 * enabled the browser to accept cookies or not. The only way to test if
 * cookies are accepted is to set one with setcookie,
 * reload, and check for the value.
 */
#[Pure(true)]
function get_browser(null|string $user_agent, bool $return_array = false): object|array|false
{
}

/**
 * @pure
 */
function crypt(string $string, string $salt): string
{
}

/**
 * @param null|resource $context
 *
 * @return resource|false
 */
function opendir(string $directory, $context = null)
{
}

/**
 * @param null|resource $dir_handle
 */
function closedir($dir_handle = null): void
{
}

/**
 * Change directory
 * @link https://php.net/manual/en/function.chdir.php
 * @param string $directory <p>
 * The new current directory
 * </p>
 * @return bool true on success or false on failure.
 */
function chdir(string $directory): bool
{
}

/**
 * Change the root directory
 * @link https://php.net/manual/en/function.chroot.php
 * @param string $directory <p>
 * The new directory
 * </p>
 * @return bool true on success or false on failure.
 */
function chroot(string $directory): bool
{
}

/**
 * @return non-empty-string|false
 */
function getcwd(): string|false
{
}

/**
 * @param resource $dir_handle
 */
function rewinddir($dir_handle): void
{
}

/**
 * @param resource $dir_handle
 *
 * @return non-empty-string|false
 */
function readdir($dir_handle): string|false
{
}

/**
 * @param resource $context
 */
function dir(string $directory, $context): Directory|false
{
}

/**
 * @param resource $context
 */
function getdir(string $directory, $context = null): Directory|false
{
}

/**
 * List files and directories inside the specified path
 * @link https://php.net/manual/en/function.scandir.php
 * @param string $directory <p>
 * The directory that will be scanned.
 * </p>
 * @param int $sorting_order <p>
 * By default, the sorted order is alphabetical in ascending order. If
 * the optional sorting_order is set to non-zero,
 * then the sort order is alphabetical in descending order.
 * </p>
 * @param resource $context [optional] <p>
 * For a description of the context parameter,
 * refer to the streams section of
 * the manual.
 * </p>
 * @return array|false an array of filenames on success, or false on
 * failure. If directory is not a directory, then
 * boolean false is returned, and an error of level
 * E_WARNING is generated.
 */
function scandir(string $directory, int $sorting_order = 0, $context): array|false
{
}

/**
 * Find pathnames matching a pattern
 * @link https://php.net/manual/en/function.glob.php
 * @param string $pattern <p>
 * The pattern. No tilde expansion or parameter substitution is done.
 * </p>
 * @param int $flags <p>
 * Valid flags:
 * GLOB_MARK - Adds a slash to each directory returned
 * GLOB_NOSORT - Return files as they appear in the directory (no sorting). When this flag is not used, the pathnames are sorted alphabetically
 * GLOB_NOCHECK - Return the search pattern if no files matching it were found
 * GLOB_NOESCAPE - Backslashes do not quote metacharacters
 * GLOB_BRACE - Expands {a,b,c} to match 'a', 'b', or 'c'
 * GLOB_ONLYDIR - Return only directory entries which match the pattern
 * GLOB_ERR - Stop on read errors (like unreadable directories), by default errors are ignored.
 * @return array|false an array containing the matched files/directories, an empty array
 * if no file matched or false on error.
 * </p>
 * <p>
 * On some systems it is impossible to distinguish between empty match and an
 * error.</p>
 */
#[Pure(true)]
function glob(string $pattern, int $flags = 0): array|false
{
}

/**
 * @return int<1750595956, max>|false
 */
function fileatime(string $filename): int|false
{
}

/**
 * @return int<1750595956, max>|false
 */
function filectime(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function filegroup(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function fileinode(string $filename): int|false
{
}

/**
 * @return int<1750595956, max>|false
 */
function filemtime(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function fileowner(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function fileperms(string $filename): int|false
{
}

/**
 * @return int<0, max>|false
 */
function filesize(string $filename): int|false
{
}

/**
 * @return 'fifo'|'char'|'dir'|'block'|'link'|'file'|'socket'|'unknown'|false
 */
function filetype(string $filename): string|false
{
}

function file_exists(string $filename): bool
{
}

function is_writable(string $filename): bool
{
}

function is_writeable(string $filename): bool
{
}

function is_readable(string $filename): bool
{
}

function is_executable(string $filename): bool
{
}

function is_file(string $filename): bool
{
}

function is_dir(string $filename): bool
{
}

function is_link(string $filename): bool
{
}

/**
 * @return array{
 *   'dev': int,
 *   'ino': int,
 *   'mode': int,
 *   'nlink': int,
 *   'uid': int,
 *   'gid': int,
 *   'rdev': int,
 *   'size': int,
 *   'atime': int,
 *   'mtime': int,
 *   'ctime': int,
 *   'blksize': int,
 *   'blocks': int,
 * }|false
 */
function stat(string $filename): array|false
{
}

/**
 * Gives information about a file or symbolic link
 * @link https://php.net/manual/en/function.lstat.php
 * @see stat
 * @param string $filename <p>
 * Path to a file or a symbolic link.
 * </p>
 * @return array|false See the manual page for stat for information on
 * the structure of the array that lstat returns.
 * This function is identical to the stat function
 * except that if the filename parameter is a symbolic
 * link, the status of the symbolic link is returned, not the status of the
 * file pointed to by the symbolic link.
 */
#[Pure(true)]
function lstat(string $filename): array|false
{
}

function chown(string $filename, string|int $user): bool
{
}

function chgrp(string $filename, string|int $group): bool
{
}

function lchown(string $filename, string|int $user): bool
{
}

function lchgrp(string $filename, string|int $group): bool
{
}

function chmod(string $filename, int $permissions): bool
{
}

function touch(string $filename, null|int $mtime = null, null|int $atime = null): bool
{
}

function clearstatcache(bool $clear_realpath_cache = false, string $filename = ''): void
{
}

function disk_total_space(string $directory): float|false
{
}

function disk_free_space(string $directory): float|false
{
}

function diskfreespace(string $directory): float|false
{
}

/**
 * Send mail
 * @link https://php.net/manual/en/function.mail.php
 * @param string $to <p>
 * Receiver, or receivers of the mail.
 * </p>
 * <p>
 * The formatting of this string must comply with
 * RFC 2822. Some examples are:
 * user@example.com
 * user@example.com, anotheruser@example.com
 * User &lt;user@example.com&gt;
 * User &lt;user@example.com&gt;, Another User &lt;anotheruser@example.com&gt;
 * </p>
 * @param string $subject <p>
 * Subject of the email to be sent.
 * </p>
 * <p>
 * Subject must satisfy RFC 2047.
 * </p>
 * @param string $message <p>
 * Message to be sent.
 * </p>
 * <p>
 * Each line should be separated with a LF (\n). Lines should not be larger
 * than 70 characters.
 * </p>
 * <p>
 * <strong>Caution</strong>
 * (Windows only) When PHP is talking to a SMTP server directly, if a full
 * stop is found on the start of a line, it is removed. To counter-act this,
 * replace these occurrences with a double dot.
 * </p>
 * <pre>
 * <?php
 * $text = str_replace("\n.", "\n..", $text);
 * ?>
 * </pre>
 * @param string|array $additional_headers <p>
 * String or array to be inserted at the end of the email header.<br/>
 * Since 7.2.0 accepts an array. Its keys are the header names and its values are the respective header values.
 * </p>
 * <p>
 * This is typically used to add extra headers (From, Cc, and Bcc).
 * Multiple extra headers should be separated with a CRLF (\r\n).
 * </p>
 * <p>
 * When sending mail, the mail must contain
 * a From header. This can be set with the
 * additional_headers parameter, or a default
 * can be set in "php.ini".
 * </p>
 * <p>
 * Failing to do this will result in an error
 * message similar to Warning: mail(): "sendmail_from" not
 * set in php.ini or custom "From:" header missing.
 * The From header sets also
 * Return-Path under Windows.
 * </p>
 * <p>
 * If messages are not received, try using a LF (\n) only.
 * Some poor quality Unix mail transfer agents replace LF by CRLF
 * automatically (which leads to doubling CR if CRLF is used).
 * This should be a last resort, as it does not comply with
 * RFC 2822.
 * </p>
 * @param string $additional_params <p>
 * The additional_parameters parameter
 * can be used to pass additional flags as command line options to the
 * program configured to be used when sending mail, as defined by the
 * sendmail_path configuration setting. For example,
 * this can be used to set the envelope sender address when using
 * sendmail with the -f sendmail option.
 * </p>
 * <p>
 * The user that the webserver runs as should be added as a trusted user to the
 * sendmail configuration to prevent a 'X-Warning' header from being added
 * to the message when the envelope sender (-f) is set using this method.
 * For sendmail users, this file is /etc/mail/trusted-users.
 * </p>
 * @return bool true if the mail was successfully accepted for delivery, false otherwise.
 * <p>
 * It is important to note that just because the mail was accepted for delivery,
 * it does NOT mean the mail will actually reach the intended destination.
 * </p>
 */
function mail(
    string $to,
    string $subject,
    string $message,
    array|string $additional_headers = [],
    string $additional_params = '',
): bool {
}

function openlog(string $prefix, int $flags, int $facility): bool
{
}
