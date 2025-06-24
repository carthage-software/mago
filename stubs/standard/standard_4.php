<?php

use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Deprecated;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Pure;

/**
 * @return array{type: int, message: string, file: string, line: int}|null
 *
 * @pure
 */
function error_get_last(): null|array
{
}

/**
 * @tempalte I
 * @template R
 *
 * @template (callble(...I): R) $callback
 *
 * @param I ...$args
 *
 * @return R
 */
function call_user_func(callable $callback, mixed ...$args): mixed
{
}

/**
 * @tempalte I
 * @template R
 *
 * @template (callble(...I): R) $callback
 *
 * @param array<I> $args
 *
 * @return R
 */
function call_user_func_array(callable $callback, array $args): mixed
{
}

/**
 * Call a static method
 * @link https://php.net/manual/en/function.forward-static-call.php
 * @param callable $callback <p>
 * The function or method to be called. This parameter may be an array,
 * with the name of the class, and the method, or a string, with a function
 * name.
 * </p>
 * @param mixed ...$args [optional] <p>
 * Zero or more parameters to be passed to the function.
 * </p>
 * @return mixed the function result, or false on error.
 */
function forward_static_call(callable $callback, mixed ...$args): mixed
{
}

/**
 * Call a static method and pass the arguments as array
 * @link https://php.net/manual/en/function.forward-static-call-array.php
 * @param callable $callback <p>
 * The function or method to be called. This parameter may be an array,
 * with the name of the class, and the method, or a string, with a function
 * name.
 * </p>
 * @param array $args
 * @return mixed the function result, or false on error.
 */
function forward_static_call_array(callable $callback, array $args): mixed
{
}

/**
 * @return non-empty-string
 */
function serialize(mixed $value): string
{
}

function unserialize(string $data, array $options = []): mixed
{
}

function var_dump(mixed $value, mixed ...$values): void
{
}

/**
 * @return ($return is true ? non-empty-string : null)
 */
function var_export(mixed $value, bool $return = false): null|string
{
}

function debug_zval_dump(mixed $value, mixed ...$values): void
{
}

/**
 * @return ($return is true ? non-empty-string : bool)
 */
function print_r(mixed $value, bool $return = false): string|bool
{
}

function memory_get_usage(bool $real_usage = false): int
{
}

function memory_get_peak_usage(bool $real_usage = false): int
{
}

function memory_reset_peak_usage(): void
{
}

/**
 * Register a function for execution on shutdown
 * @link https://php.net/manual/en/function.register-shutdown-function.php
 * @param callable $callback <p>
 * The shutdown function to register.
 * </p>
 * <p>
 * The shutdown functions are called as the part of the request so that
 * it's possible to send the output from them. There is currently no way
 * to process the data with output buffering functions in the shutdown
 * function.
 * </p>
 * <p>
 * Shutdown functions are called after closing all opened output buffers
 * thus, for example, its output will not be compressed if zlib.output_compression is
 * enabled.
 * </p>
 * @param mixed ...$args [optional] <p>
 * It is possible to pass parameters to the shutdown function by passing
 * additional parameters.
 * </p>
 * @return bool|null
 */
function register_shutdown_function(callable $callback, mixed ...$args): void
{
}

/**
 * Register a function for execution on each tick
 * @link https://php.net/manual/en/function.register-tick-function.php
 * @param callable $callback <p>
 * The function name as a string, or an array consisting of an object and
 * a method.
 * </p>
 * @param mixed ...$args [optional] <p>
 * </p>
 * @return bool true on success or false on failure.
 */
function register_tick_function(callable $callback, mixed ...$args): bool
{
}

/**
 * De-register a function for execution on each tick
 * @link https://php.net/manual/en/function.unregister-tick-function.php
 * @param callable $callback <p>
 * The function name as a string, or an array consisting of an object and
 * a method.
 * </p>
 * @return void
 */
function unregister_tick_function(callable $callback): void
{
}

/**
 * @return ($return is true ? string|false : bool)
 */
function highlight_file(string $filename, bool $return = false): string|bool
{
}

/**
 * @return ($return is true ? string|false : bool)
 */
function show_source(string $filename, bool $return = false): string|bool
{
}

/**
 * @return ($return is true ? string|false : bool)
 */
function highlight_string(string $string, bool $return = false): string|bool
{
}

/**
 * @return ($as_number is true ? int|float|false : list{int, int}|false)
 *
 * @mutation-free
 */
function hrtime(bool $as_number = false): array|int|float|false
{
}

function php_strip_whitespace(string $filename): string
{
}

/**
 * Gets the value of a configuration option
 * @link https://php.net/manual/en/function.ini-get.php
 * @link https://php.net/manual/en/ini.list.php
 * @param string $option <p>
 * The configuration option name.
 * </p>
 * @return string|false the value of the configuration option as a string on success, or
 * an empty string on failure or for null values.
 */
#[Pure(true)]
function ini_get(string $option): string|false
{
}

/**
 * @return array{'global_value': string, 'local_value': string, 'access': int}|false
 */
function ini_get_all(null|string $extension, bool $details = true): array|false
{
}

function ini_set(string $option, string|int|float|bool|null $value): string|false
{
}

function ini_alter(string $option, string|int|float|bool|null $value): string|false
{
}

function ini_restore(string $option): void
{
}

function ini_parse_quantity(string $shorthand): int
{
}

function get_include_path(): string|false
{
}

function set_include_path(string $include_path): string|false
{
}

/**
 * Send a cookie
 * @link https://php.net/manual/en/function.setcookie.php
 * @param string $name <p>
 * The name of the cookie.
 * </p>
 * @param string $value [optional] <p>
 * The value of the cookie. This value is stored on the clients
 * computer; do not store sensitive information.
 * Assuming the name is 'cookiename', this
 * value is retrieved through $_COOKIE['cookiename']
 * </p>
 * @param int $expires_or_options [optional] <p>
 * The time the cookie expires. This is a Unix timestamp so is
 * in number of seconds since the epoch. In other words, you'll
 * most likely set this with the time function
 * plus the number of seconds before you want it to expire. Or
 * you might use mktime.
 * time()+60*60*24*30 will set the cookie to
 * expire in 30 days. If set to 0, or omitted, the cookie will expire at
 * the end of the session (when the browser closes).
 * </p>
 * <p>
 * <p>
 * You may notice the expire parameter takes on a
 * Unix timestamp, as opposed to the date format Wdy, DD-Mon-YYYY
 * HH:MM:SS GMT, this is because PHP does this conversion
 * internally.
 * </p>
 * <p>
 * expire is compared to the client's time which can
 * differ from server's time.
 * </p>
 * </p>
 * @param string $path [optional] <p>
 * The path on the server in which the cookie will be available on.
 * If set to '/', the cookie will be available
 * within the entire domain. If set to
 * '/foo/', the cookie will only be available
 * within the /foo/ directory and all
 * sub-directories such as /foo/bar/ of
 * domain. The default value is the
 * current directory that the cookie is being set in.
 * </p>
 * @param string $domain [optional] <p>
 * The domain that the cookie is available.
 * To make the cookie available on all subdomains of example.com
 * then you'd set it to '.example.com'. The
 * . is not required but makes it compatible
 * with more browsers. Setting it to www.example.com
 * will make the cookie only available in the www
 * subdomain. Refer to tail matching in the
 * spec for details.
 * </p>
 * @param bool $secure [optional] <p>
 * Indicates that the cookie should only be transmitted over a
 * secure HTTPS connection from the client. When set to true, the
 * cookie will only be set if a secure connection exists.
 * On the server-side, it's on the programmer to send this
 * kind of cookie only on secure connection (e.g. with respect to
 * $_SERVER["HTTPS"]).
 * </p>
 * @param bool $httponly [optional] <p>
 * When true the cookie will be made accessible only through the HTTP
 * protocol. This means that the cookie won't be accessible by
 * scripting languages, such as JavaScript. This setting can effectively
 * help to reduce identity theft through XSS attacks (although it is
 * not supported by all browsers). Added in PHP 5.2.0.
 * true or false
 * </p>
 * @return bool If output exists prior to calling this function,
 * setcookie will fail and return false. If
 * setcookie successfully runs, it will return true.
 * This does not indicate whether the user accepted the cookie.
 */
function setcookie(
    string $name,
    string $value = '',
    int $expires_or_options = 0,
    string $path = '',
    string $domain = '',
    bool $secure = false,
    bool $httponly = false,
): bool {
}

/**
 * Send a cookie
 *
 * @link  https://php.net/manual/en/function.setcookie.php
 *
 * @param string $name The name of the cookie.
 * @param string $value [optional] The value of the cookie. This value is stored on the clients
 *                        computer; do not store sensitive information.
 *                        Assuming the name is 'cookiename', this value is retrieved through $_COOKIE['cookiename']
 * @param array $options [optional] An associative array which may have any of the keys expires, path, domain, secure,
 *                        httponly and samesite. The values have the same meaning as described for the parameters with
 *                        the same name. The value of the samesite element should be either Lax or Strict.
 *                        If any of the allowed options are not given, their default values are the same
 *                        as the default values of the explicit parameters. If the samesite element is omitted,
 *                        no SameSite cookie attribute is set.
 *
 * @return bool           If output exists prior to calling this function, setcookie will fail and return false. If
 *                        setcookie successfully runs, it will return true.
 *                        This does not indicate whether the user accepted the cookie.
 * @since 7.3
 */
function setcookie(string $name, string $value = '', array $options = []): bool
{
}

/**
 * Send a cookie without urlencoding the cookie value
 * @link https://php.net/manual/en/function.setrawcookie.php
 * @param string $name
 * @param string $value [optional]
 * @param int $expires_or_options [optional]
 * @param string $path [optional]
 * @param string $domain [optional]
 * @param bool $secure [optional]
 * @param bool $httponly [optional]
 * @return bool true on success or false on failure.
 */
function setrawcookie(
    string $name,
    $value = '',
    $expires_or_options = 0,
    $path = '',
    $domain = '',
    $secure = false,
    $httponly = false,
): bool {
}

/**
 * Send a cookie without urlencoding the cookie value
 *
 * @link https://php.net/manual/en/function.setrawcookie.php
 *
 * @param string $name The name of the cookie.
 * @param string $value [optional] The value of the cookie. This value is stored on the clients
 *                        computer; do not store sensitive information.
 *                        Assuming the name is 'cookiename', this value is retrieved through $_COOKIE['cookiename']
 * @param array $options [optional] An associative array which may have any of the keys expires, path, domain, secure,
 *                        httponly and samesite. The values have the same meaning as described for the parameters with
 *                        the same name. The value of the samesite element should be either Lax or Strict.
 *                        If any of the allowed options are not given, their default values are the same
 *                        as the default values of the explicit parameters. If the samesite element is omitted,
 *                        no SameSite cookie attribute is set.
 *
 * @return bool           If output exists prior to calling this function, setcookie will fail and return false. If
 *                        setcookie successfully runs, it will return true.
 *                        This does not indicate whether the user accepted the cookie.
 * @since 7.3
 */
function setrawcookie(string $name, $value = '', array $options = []): bool
{
}

/**
 * Send a raw HTTP header
 * @link https://php.net/manual/en/function.header.php
 * @param string $header <p>
 * The header string.
 * </p>
 * <p>
 * There are two special-case header calls. The first is a header
 * that starts with the string "HTTP/" (case is not
 * significant), which will be used to figure out the HTTP status
 * code to send. For example, if you have configured Apache to
 * use a PHP script to handle requests for missing files (using
 * the ErrorDocument directive), you may want to
 * make sure that your script generates the proper status code.
 * </p>
 * <p>
 * The second special case is the "Location:" header. Not only does
 * it send this header back to the browser, but it also returns a
 * REDIRECT (302) status code to the browser
 * unless the 201 or
 * a 3xx status code has already been set.
 * </p>
 * @param bool $replace [optional] <p>
 * The optional replace parameter indicates
 * whether the header should replace a previous similar header, or
 * add a second header of the same type. By default it will replace,
 * but if you pass in false as the second argument you can force
 * multiple headers of the same type. For example:
 * </p>
 * @param int $response_code <p>
 * Forces the HTTP response code to the specified value.
 * </p>
 * @return void
 */
function header(string $header, bool $replace = true, int $response_code = 0): void
{
}

/**
 * Remove previously set headers
 * @link https://php.net/manual/en/function.header-remove.php
 * @param string|null $name [optional] <p>
 * The header name to be removed.
 * </p>
 * This parameter is case-insensitive.
 * @return void
 */
function header_remove(null|string $name = null): void
{
}

/**
 * Checks if or where headers have been sent
 * @link https://php.net/manual/en/function.headers-sent.php
 * @param string &$filename [optional] <p>
 * If the optional file and
 * line parameters are set,
 * headers_sent will put the PHP source file name
 * and line number where output started in the file
 * and line variables.
 * </p>
 * @param int &$line [optional] <p>
 * The line number where the output started.
 * </p>
 * @return bool headers_sent will return false if no HTTP headers
 * have already been sent or true otherwise.
 */
function headers_sent(&$filename = null, &$line = null): bool
{
}

/**
 * Returns a list of response headers sent (or ready to send)
 * @link https://php.net/manual/en/function.headers-list.php
 * @return array a numerically indexed array of headers.
 */
#[Pure]
function headers_list(): array
{
}

/**
 * Fetches all HTTP request headers from the current request
 * @link https://php.net/manual/en/function.apache-request-headers.php
 * @return array|false An associative array of all the HTTP headers in the current request, or <b>FALSE</b> on failure.
 */
#[Pure]
function apache_request_headers(): false|array
{
}

/**
 * Fetches all HTTP headers from the current request.
 * This function is an alias for apache_request_headers(). Please read the apache_request_headers() documentation for more information on how this function works.
 * @link https://php.net/manual/en/function.getallheaders.php
 * @return array|false An associative array of all the HTTP headers in the current request, or <b>FALSE</b> on failure.
 */
#[Pure]
function getallheaders(): false|array
{
}

/**
 * Check whether client disconnected
 * @link https://php.net/manual/en/function.connection-aborted.php
 * @return int 1 if client disconnected, 0 otherwise.
 */
#[Pure(true)]
function connection_aborted(): int
{
}

/**
 * Returns connection status bitfield
 * @link https://php.net/manual/en/function.connection-status.php
 * @return int the connection status bitfield, which can be used against the
 * CONNECTION_XXX constants to determine the connection
 * status.
 */
#[Pure(true)]
function connection_status(): int
{
}

/**
 * Set whether a client disconnect should abort script execution
 * @link https://php.net/manual/en/function.ignore-user-abort.php
 * @param bool|null $enable [optional] <p>
 * If set, this function will set the ignore_user_abort ini setting
 * to the given value. If not, this function will
 * only return the previous setting without changing it.
 * </p>
 * @return int the previous setting, as an integer.
 */
function ignore_user_abort(null|bool $enable): int
{
}

/**
 * Parse a configuration file
 * @link https://php.net/manual/en/function.parse-ini-file.php
 * @param string $filename <p>
 * The filename of the ini file being parsed.
 * </p>
 * @param bool $process_sections [optional] <p>
 * By setting the process_sections
 * parameter to true, you get a multidimensional array, with
 * the section names and settings included. The default
 * for process_sections is false
 * </p>
 * @param int $scanner_mode [optional] <p>
 * Can either be INI_SCANNER_NORMAL (default) or
 * INI_SCANNER_RAW. If INI_SCANNER_RAW
 * is supplied, then option values will not be parsed.
 * </p>
 * <p>
 * As of PHP 5.6.1 can also be specified as <strong><code>INI_SCANNER_TYPED</code></strong>.
 * In this mode boolean, null and integer types are preserved when possible.
 * String values <em>"true"</em>, <em>"on"</em> and <em>"yes"</em>
 * are converted to <b>TRUE</b>. <em>"false"</em>, <em>"off"</em>, <em>"no"</em>
 * and <em>"none"</em> are considered <b>FALSE</b>. <em>"null"</em> is converted to <b>NULL</b>
 * in typed mode. Also, all numeric strings are converted to integer type if it is possible.
 * </p>
 * @return array|false The settings are returned as an associative array on success,
 * and false on failure.
 */
#[Pure(true)]
function parse_ini_file(
    string $filename,
    bool $process_sections = false,
    int $scanner_mode = INI_SCANNER_NORMAL,
): array|false {
}

/**
 * Parse a configuration string
 * @link https://php.net/manual/en/function.parse-ini-string.php
 * @param string $ini_string <p>
 * The contents of the ini file being parsed.
 * </p>
 * @param bool $process_sections [optional] <p>
 * By setting the process_sections
 * parameter to true, you get a multidimensional array, with
 * the section names and settings included. The default
 * for process_sections is false
 * </p>
 * @param int $scanner_mode [optional] <p>
 * Can either be INI_SCANNER_NORMAL (default) or
 * INI_SCANNER_RAW. If INI_SCANNER_RAW
 * is supplied, then option values will not be parsed.
 * </p>
 * @return array|false The settings are returned as an associative array on success,
 * and false on failure.
 */
#[Pure]
function parse_ini_string(
    string $ini_string,
    bool $process_sections = false,
    int $scanner_mode = INI_SCANNER_NORMAL,
): array|false {
}

/**
 * Tells whether the file was uploaded via HTTP POST
 * @link https://php.net/manual/en/function.is-uploaded-file.php
 * @param string $filename <p>
 * The filename being checked.
 * </p>
 * @return bool true on success or false on failure.
 */
#[Pure(true)]
function is_uploaded_file(string $filename): bool
{
}

/**
 * Moves an uploaded file to a new location
 * @link https://php.net/manual/en/function.move-uploaded-file.php
 * @param string $from <p>
 * The filename of the uploaded file.
 * </p>
 * @param string $to <p>
 * The destination of the moved file.
 * </p>
 * @return bool If filename is not a valid upload file,
 * then no action will occur, and
 * move_uploaded_file will return
 * false.
 * </p>
 * <p>
 * If filename is a valid upload file, but
 * cannot be moved for some reason, no action will occur, and
 * move_uploaded_file will return
 * false. Additionally, a warning will be issued.
 */
function move_uploaded_file(string $from, string $to): bool
{
}

/**
 * @return array|false
 * @since 7.3
 */
#[Pure]
#[ArrayShape(['description' => 'string', 'mac' => 'string', 'mtu' => 'int', 'unicast' => 'array', 'up' => 'bool'])]
function net_get_interfaces(): array|false
{
}

/**
 * Get the Internet host name corresponding to a given IP address
 * @link https://php.net/manual/en/function.gethostbyaddr.php
 * @param string $ip <p>
 * The host IP address.
 * </p>
 * @return string|false the host name or the unmodified ip_address
 * on failure.
 */
#[Pure]
function gethostbyaddr(string $ip): string|false
{
}

/**
 * Get the IPv4 address corresponding to a given Internet host name
 * @link https://php.net/manual/en/function.gethostbyname.php
 * @param string $hostname <p>
 * The host name.
 * </p>
 * @return string the IPv4 address or a string containing the unmodified
 * hostname on failure.
 */
#[Pure]
function gethostbyname(string $hostname): string
{
}

/**
 * Get a list of IPv4 addresses corresponding to a given Internet host
 * name
 * @link https://php.net/manual/en/function.gethostbynamel.php
 * @param string $hostname <p>
 * The host name.
 * </p>
 * @return array|false an array of IPv4 addresses or false if
 * hostname could not be resolved.
 */
#[Pure]
function gethostbynamel(string $hostname): array|false
{
}

/**
 * Gets the host name
 * @link https://php.net/manual/en/function.gethostname.php
 * @return string|false a string with the hostname on success, otherwise false is
 * returned.
 */
#[Pure]
function gethostname(): string|false
{
}

/**
 * Alias:
 * {@see checkdnsrr}
 * @link https://php.net/manual/en/function.dns-check-record.php
 * @param string $hostname <p>
 * <b>host</b> may either be the IP address in
 * dotted-quad notation or the host name.
 * </p>
 * @param string $type [optional] <p>
 * <b>type</b> may be any one of: A, MX, NS, SOA,
 * PTR, CNAME, AAAA, A6, SRV, NAPTR, TXT or ANY.
 * </p>
 * @return bool Returns <b>TRUE</b> if any records are found; returns <b>FALSE</b> if no records were found or if an error occurred.
 */
function dns_check_record(string $hostname, string $type = 'MX'): bool
{
}

/**
 * Check DNS records corresponding to a given Internet host name or IP address
 * @link https://php.net/manual/en/function.checkdnsrr.php
 * @param string $hostname <p>
 * host may either be the IP address in
 * dotted-quad notation or the host name.
 * </p>
 * @param string $type [optional] <p>
 * type may be any one of: A, MX, NS, SOA,
 * PTR, CNAME, AAAA, A6, SRV, NAPTR, TXT or ANY.
 * </p>
 * @return bool true if any records are found; returns false if no records
 * were found or if an error occurred.
 */
#[Pure]
function checkdnsrr(string $hostname, string $type = 'MX'): bool
{
}

/**
 * Alias:
 * {@see getmxrr}
 * @link https://php.net/manual/en/function.dns-get-mx.php
 * @param string $hostname
 * @param array &$hosts
 * @param array &$weights [optional]
 * @return bool
 */
function dns_get_mx(string $hostname, &$hosts, &$weights): bool
{
}

/**
 * Get MX records corresponding to a given Internet host name
 * @link https://php.net/manual/en/function.getmxrr.php
 * @param string $hostname <p>
 * The Internet host name.
 * </p>
 * @param array &$hosts <p>
 * A list of the MX records found is placed into the array
 * mxhosts.
 * </p>
 * @param array &$weights [optional] <p>
 * If the weight array is given, it will be filled
 * with the weight information gathered.
 * </p>
 * @return bool true if any records are found; returns false if no records
 * were found or if an error occurred.
 */
function getmxrr(string $hostname, &$hosts, &$weights): bool
{
}

/**
 * Fetch DNS Resource Records associated with a hostname
 * @link https://php.net/manual/en/function.dns-get-record.php
 * @param string $hostname <p>
 * hostname should be a valid DNS hostname such
 * as "www.example.com". Reverse lookups can be generated
 * using in-addr.arpa notation, but
 * gethostbyaddr is more suitable for
 * the majority of reverse lookups.
 * </p>
 * <p>
 * Per DNS standards, email addresses are given in user.host format (for
 * example: hostmaster.example.com as opposed to hostmaster@example.com),
 * be sure to check this value and modify if necessary before using it
 * with a functions such as mail.
 * </p>
 * @param int $type [optional] <p>
 * By default, dns_get_record will search for any
 * resource records associated with hostname.
 * To limit the query, specify the optional type
 * parameter. May be any one of the following:
 * DNS_A, DNS_CNAME,
 * DNS_HINFO, DNS_MX,
 * DNS_NS, DNS_PTR,
 * DNS_SOA, DNS_TXT,
 * DNS_AAAA, DNS_SRV,
 * DNS_NAPTR, DNS_A6,
 * DNS_ALL or DNS_ANY.
 * </p>
 * <p>
 * Because of eccentricities in the performance of libresolv
 * between platforms, DNS_ANY will not
 * always return every record, the slower DNS_ALL
 * will collect all records more reliably.
 * </p>
 * @param array &$authoritative_name_servers [optional] <p>
 * Passed by reference and, if given, will be populated with Resource
 * Records for the Authoritative Name Servers.
 * </p>
 * @param array &$additional_records [optional] <p>
 * Passed by reference and, if given, will be populated with any
 * Additional Records. *
 */
function dns_get_record(
    string $hostname,
    int $type = DNS_ANY,
    &$authoritative_name_servers = null,
    &$additional_records = null,
    bool $raw = false,
): array|false {
}
