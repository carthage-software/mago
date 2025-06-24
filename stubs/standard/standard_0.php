<?php

final class __PHP_Incomplete_Class
{
    /**
     * @var string
     */
    public $__PHP_Incomplete_Class_Name;
}

class php_user_filter
{
    public string $filtername;
    public mixed $params;
    public $stream;

    /**
     * @param resource $in
     * @param resource $out
     * @param int &$consumed
     */
    public function filter($in, $out, &$consumed, bool $closing): int
    {
    }

    public function onCreate(): bool
    {
    }

    public function onClose(): void
    {
    }
}

final class StreamBucket
{
    public $bucket;
    public string $data;
    public int $datalen;
    public int $dataLength;
}

class Directory
{
    public readonly string $path;

    /**
     * @var resource
     */
    public readonly mixed $handle;

    public function close(): void
    {
    }

    public function rewind(): void
    {
    }

    public function read(): string|false
    {
    }
}

/**
 * @throws Error
 *
 * @pure
 */
function constant(string $name): mixed
{
}

/**
 * @pure
 */
function bin2hex(string $string): string
{
}

/**
 * @param int<0, max> $seconds
 */
function sleep(int $seconds): int
{
}

/**
 * @param int<0, max> $microseconds
 */
function usleep(int $microseconds): void
{
}

/**
 * @param positive-int $seconds
 * @param positive-int $nanoseconds
 *
 * @return bool|array{seconds: int, nanoseconds: int}
 */
function time_nanosleep(int $seconds, int $nanoseconds): array|bool
{
}

function time_sleep_until(float $timestamp): bool
{
}

/**
 * @return false|array{tm_sec: int, tm_min: int, tm_hour: int, tm_mday: int, tm_mon: int, tm_year: int, tm_wday: int, tm_yday: int, unparsed: string}
 */
function strptime(string $timestamp, string $format): array|false
{
}

function flush(): void
{
}

/**
 * @pure
 */
function wordwrap(string $string, int $width = 75, string $break = "\n", bool $cut_long_words = false): string
{
}

/**
 * @pure
 */
function htmlspecialchars(
    string $string,
    int $flags = ENT_QUOTES | ENT_SUBSTITUTE,
    null|string $encoding = null,
    bool $double_encode = true,
): string {
}

/**
 * @pure
 */
function htmlentities(
    string $string,
    int $flags = ENT_QUOTES | ENT_SUBSTITUTE,
    null|string $encoding,
    bool $double_encode = true,
): string {
}

/**
 * @pure
 */
function html_entity_decode(string $string, int $flags = ENT_QUOTES | ENT_SUBSTITUTE, null|string $encoding): string
{
}

/**
 * @pure
 */
function htmlspecialchars_decode(string $string, int $flags = ENT_QUOTES | ENT_SUBSTITUTE): string
{
}

/**
 * @pure
 */
function get_html_translation_table(
    int $table = 0,
    int $flags = ENT_QUOTES | ENT_SUBSTITUTE,
    string $encoding = 'UTF-8',
): array {
}

/**
 * @pure
 */
function sha1(string $string, bool $binary = false): string
{
}

function sha1_file(string $filename, bool $binary = false): string|false
{
}

/**
 * @pure
 */
function md5(string $string, bool $binary = false): string
{
}

function md5_file(string $filename, bool $binary = false): string|false
{
}

/**
 * @pure
 */
function crc32(string $string): int
{
}

/**
 * @pure
 */
function iptcparse(string $iptc_block): array|false
{
}

function iptcembed(string $iptc_data, string $filename, int $spool = 0): string|bool
{
}

/**
 * @param string $filename
 * @param array &$image_info
 *
 * @return false|array{0: int, 1: int, 2: int, 3: string, bits: int, channels: int, mime: string}
 */
#[ArrayShape([
    0 => 'int',
    1 => 'int',
    2 => 'int',
    3 => 'string',
    'bits' => 'int',
    'channels' => 'int',
    'mime' => 'string',
])]
function getimagesize(string $filename, &$image_info): array|false
{
}

/**
 * @pure
 */
function image_type_to_mime_type(int $image_type): string
{
}

/**
 * @pure
 */
function image_type_to_extension(int $image_type, bool $include_dot = true): string|false
{
}

function phpinfo(int $flags = INFO_ALL): bool
{
}

/**
 * @pure
 */
function phpversion(null|string $extension): string|false
{
}

function phpcredits(int $flags = CREDITS_ALL): bool
{
}

/**
 * @return 'cli'|'phpdbg'|'embed'|'apache'|'apache2handler'|'cgi-fcgi'|'cli-server'|'fpm-fcgi'|'litespeed'|false
 * @pure
 */
function php_sapi_name(): string|false
{
}

/**
 * @pure
 */
function php_uname(string $mode = 'a'): string
{
}

/**
 * @pure
 */
function php_ini_scanned_files(): string|false
{
}

/**
 * @pure
 */
function php_ini_loaded_file(): string|false
{
}

/**
 * @pure
 */
function strnatcmp(string $string1, string $string2): int
{
}

/**
 * @pure
 */
function strnatcasecmp(string $string1, string $string2): int
{
}

/**
 * @return int<0,max>
 *
 * @pure
 */
function substr_count(string $haystack, string $needle, int $offset = 0, null|int $length): int
{
}

/**
 * @pure
 */
function strspn(string $string, string $characters, int $offset = 0, null|int $length): int
{
}

/**
 * @pure
 */
function strcspn(string $string, string $characters, int $offset = 0, null|int $length): int
{
}

/**
 * @pure
 */
function strtok(string $string, null|string $token = null): string|false
{
}
