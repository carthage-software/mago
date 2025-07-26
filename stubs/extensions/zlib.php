<?php

/**
 * @var int
 */
const FORCE_GZIP = UNKNOWN;

/**
 * @var int
 */
const FORCE_DEFLATE = UNKNOWN;

/**
 * @var int
 */
const ZLIB_ENCODING_RAW = UNKNOWN;

/**
 * @var int
 */
const ZLIB_ENCODING_GZIP = UNKNOWN;

/**
 * @var int
 */
const ZLIB_ENCODING_DEFLATE = UNKNOWN;

/**
 * @var int
 */
const ZLIB_NO_FLUSH = UNKNOWN;

/**
 * @var int
 */
const ZLIB_PARTIAL_FLUSH = UNKNOWN;

/**
 * @var int
 */
const ZLIB_SYNC_FLUSH = UNKNOWN;

/**
 * @var int
 */
const ZLIB_FULL_FLUSH = UNKNOWN;

/**
 * @var int
 */
const ZLIB_BLOCK = UNKNOWN;

/**
 * @var int
 */
const ZLIB_FINISH = UNKNOWN;

/**
 * @var int
 */
const ZLIB_FILTERED = UNKNOWN;

/**
 * @var int
 */
const ZLIB_HUFFMAN_ONLY = UNKNOWN;

/**
 * @var int
 */
const ZLIB_RLE = UNKNOWN;

/**
 * @var int
 */
const ZLIB_FIXED = UNKNOWN;

/**
 * @var int
 */
const ZLIB_DEFAULT_STRATEGY = UNKNOWN;

/**
 * @var string
 */
const ZLIB_VERSION = UNKNOWN;

/**
 * @var int
 */
const ZLIB_VERNUM = UNKNOWN;

/**
 * @var int
 */
const ZLIB_OK = UNKNOWN;

/**
 * @var int
 */
const ZLIB_STREAM_END = UNKNOWN;

/**
 * @var int
 */
const ZLIB_NEED_DICT = UNKNOWN;

/**
 * @var int
 */
const ZLIB_ERRNO = UNKNOWN;

/**
 * @var int
 */
const ZLIB_STREAM_ERROR = UNKNOWN;

/**
 * @var int
 */
const ZLIB_DATA_ERROR = UNKNOWN;

/**
 * @var int
 */
const ZLIB_MEM_ERROR = UNKNOWN;

/**
 * @var int
 */
const ZLIB_BUF_ERROR = UNKNOWN;

/**
 * @var int
 */
const ZLIB_VERSION_ERROR = UNKNOWN;

final class InflateContext
{
}

final class DeflateContext
{
}

function ob_gzhandler(string $data, int $flags): string|false
{
}

function zlib_get_coding_type(): string|false
{
}

/**
 * @return array<int, string>|false
 */
function gzfile(string $filename, bool $use_include_path = false): array|false
{
}

/**
 * @return resource|false
 */
function gzopen(string $filename, string $mode, bool $use_include_path = false)
{
}

function readgzfile(string $filename, bool $use_include_path = false): int|false
{
}

function zlib_encode(string $data, int $encoding, int $level = -1): string|false
{
}

function zlib_decode(string $data, int $max_length = 0): string|false
{
}

function gzdeflate(string $data, int $level = -1, int $encoding = ZLIB_ENCODING_RAW): string|false
{
}

function gzencode(string $data, int $level = -1, int $encoding = ZLIB_ENCODING_GZIP): string|false
{
}

function gzcompress(string $data, int $level = -1, int $encoding = ZLIB_ENCODING_DEFLATE): string|false
{
}

function gzinflate(string $data, int $max_length = 0): string|false
{
}

function gzdecode(string $data, int $max_length = 0): string|false
{
}

function gzuncompress(string $data, int $max_length = 0): string|false
{
}

/**
 * @param resource $stream
 */
function gzwrite($stream, string $data, null|int $length = null): int|false
{
}

/**
 * @param resource $stream
 */
function gzputs($stream, string $data, null|int $length = null): int|false
{
}

/**
 * @param resource $stream
 */
function gzrewind($stream): bool
{
}

/**
 * @param resource $stream
 */
function gzclose($stream): bool
{
}

/**
 * @param resource $stream
 */
function gzeof($stream): bool
{
}

/**
 * @param resource $stream
 */
function gzgetc($stream): string|false
{
}

/**
 * @param resource $stream
 */
function gzpassthru($stream): int
{
}

/**
 * @param resource $stream
 */
function gzseek($stream, int $offset, int $whence = SEEK_SET): int
{
}

/**
 * @param resource $stream
 */
function gztell($stream): int|false
{
}

/**
 * @param resource $stream
 */
function gzread($stream, int $length): string|false
{
}

/**
 * @param resource $stream
 */
function gzgets($stream, null|int $length = null): string|false
{
}

function deflate_init(int $encoding, array|object $options = []): DeflateContext|false
{
}

function deflate_add(DeflateContext $context, string $data, int $flush_mode = ZLIB_SYNC_FLUSH): string|false
{
}

function inflate_init(int $encoding, array|object $options = []): InflateContext|false
{
}

function inflate_add(InflateContext $context, string $data, int $flush_mode = ZLIB_SYNC_FLUSH): string|false
{
}

function inflate_get_status(InflateContext $context): int
{
}

function inflate_get_read_len(InflateContext $context): int
{
}
