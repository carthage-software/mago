<?php

/**
 * @pure
 */
function boolval(mixed $value): bool
{
}

/**
 * @pure
 */
function intval(mixed $value, int $base = 10): int
{
}

/**
 * @pure
 */
function floatval(mixed $value): float
{
}

/**
 * @pure
 */
function doubleval(mixed $value): float
{
}

function strval(mixed $value): string
{
}

/**
 * @return 'boolean'|'integer'|'double'|'string'|'array'|'object'|'resource'|'NULL'|'unknown type'|'resource (closed)'
 *
 * @pure
 */
function gettype(mixed $value): string
{
}

/**
 * @param 'bool'|'boolean'|'int'|'integer'|'float'|'double'|'string'|'array'|'object'|'null' $type
 */
function settype(mixed &$var, string $type): bool
{
}

/**
 * @assert-if-true null $value
 *
 * @return ($value is null ? true : false)
 *
 * @pure
 */
function is_null(mixed $value): bool
{
}

/**
 * @assert-if-true resource $value
 *
 * @return ($value is resource ? true : false)
 *
 * @pure
 */
function is_resource(mixed $value): bool
{
}

/**
 * @assert-if-true bool $value
 *
 * @return ($value is bool ? true : false)
 *
 * @pure
 */
function is_bool(mixed $value): bool
{
}

/**
 * @assert-if-true int $value
 *
 * @return ($value is int ? true : false)
 *
 * @pure
 */
function is_long(mixed $value): bool
{
}

/**
 * @assert-if-true float $value
 *
 * @return ($value is float ? true : false)
 *
 * @pure
 */
function is_float(mixed $value): bool
{
}

/**
 * @assert-if-true int $value
 *
 * @return ($value is int ? true : false)
 *
 * @pure
 */
function is_int(mixed $value): bool
{
}

/**
 * @assert-if-true int $value
 *
 * @return ($value is int ? true : false)
 *
 * @pure
 */
function is_integer(mixed $value): bool
{
}

/**
 * @assert-if-true float $value
 *
 * @return ($value is float ? true : false)
 *
 * @pure
 */
function is_double(mixed $value): bool
{
}

/**
 * @assert-if-true float $value
 *
 * @return ($value is float ? true : false)
 *
 * @pure
 * @deprecated
 */
function is_real(mixed $var): bool
{
}

/**
 * @assert-if-true numeric $value
 *
 * @return ($value is numeric ? true : false)
 *
 * @pure
 */
function is_numeric(mixed $value): bool
{
}

/**
 * @assert-if-true string $value
 *
 * @return ($value is string ? true : false)
 *
 * @pure
 */
function is_string(mixed $value): bool
{
}

/**
 * @assert-if-true array<array-key, mixed> $value
 *
 * @return ($value is array ? true : false)
 *
 * @pure
 */
function is_array(mixed $value): bool
{
}

/**
 * @assert-if-true list<mixed> $array
 *
 * @return ($array is list ? true : false)
 *
 * @pure
 */
function array_is_list(array $array): bool
{
}

/**
 * @assert-if-true iterable $values
 *
 * @return ($values is iterable ? true : false)
 *
 * @pure
 */
function is_iterable(mixed $value): bool
{
}

/**
 * @assert-if-true object $value
 *
 * @return ($value is object ? true : false)
 *
 * @pure
 */
function is_object(mixed $value): bool
{
}

/**
 * @assert-if-true scalar $value
 *
 * @return ($value is scalar ? true : false)
 *
 * @pure
 */
function is_scalar(mixed $value): bool
{
}

/**
 * @param mixed $value
 * @param bool $syntax_only
 *
 * @param-out string $callable_name
 *
 * @assert-if-true callable $value
 *
 * @pure
 */
function is_callable(mixed $value, bool $syntax_only = false, &$callable_name = null): bool
{
}

/**
 * @pure
 */
function is_countable(mixed $value): bool
{
}

/**
 * @param resource $handle
 *
 * @return int<-1, max>
 */
function pclose($handle): int
{
}

/**
 * @return open-resource|false
 */
function popen(string $command, string $mode)
{
}

/**
 * @param null|resource $context
 */
function readfile(string $filename, bool $use_include_path = false, $context = null): int|false
{
}

/**
 * @param resource $stream
 */
function rewind($stream): bool
{
}

/**
 * @param null|resource $context
 */
function rmdir(string $directory, $context = null): bool
{
}

function umask(null|int $mask): int
{
}

/**
 * @param resource $stream
 *
 * @assert-if-true closed-resource $stream
 */
function fclose($stream): bool
{
}

/**
 * @param resource $stream
 */
function feof($stream): bool
{
}

/**
 * @param resource $stream
 */
function fgetc($stream): string|false
{
}

/**
 * @param resource $stream
 */
function fgets($stream, null|int $length): string|false
{
}

/**
 * @param resource $stream
 */
function fread($stream, int $length): string|false
{
}

/**
 * @param resource|null $context
 *
 * @return open-resource|false
 */
function fopen(string $filename, string $mode, bool $use_include_path = false, $context = null)
{
}

/**
 * @param resource $stream
 */
function fpassthru($stream): int
{
}

/**
 * @param resource $stream
 */
function ftruncate($stream, int $size): bool
{
}

/**
 * @param resource $stream
 *
 * @return false|array{
 *   'dev': int<0, max>,
 *   'ino': int<0, max>,
 *   'mode': int<0, max>,
 *   'nlink': int<0, max>,
 *   'uid': int<0, max>,
 *   'gid': int<0, max>,
 *   'rdev': int<0, max>,
 *   'size': int<0, max>,
 *   'atime': int<1750171087, max>,
 *   'mtime': int<1750171087, max>,
 *   'ctime': int<1750171087, max>,
 *   'blksize': int<0, max>,
 *   'blocks': int<0, max>,
 * }
 */
function fstat($stream): array|false
{
}

/**
 * @param resource $stream
 */
function fseek($stream, int $offset, int $whence = SEEK_SET): int
{
}

/**
 * @param resource $stream
 */
function ftell($stream): int|false
{
}

/**
 * @param resource $stream
 */
function fflush($stream): bool
{
}

/**
 * @param resource $stream
 */
function fsync($stream): bool
{
}

/**
 * @param resource $stream
 */
function fdatasync($stream): bool
{
}

/**
 * @param resource $stream
 *
 * @return int<0, max>|false
 */
function fwrite($stream, string $data, null|int $length): int|false
{
}

/**
 * @param resource $stream
 *
 * @return int<0, max>|false
 */
function fputs($stream, string $data, null|int $length): int|false
{
}

/**
 * @param null|resource $context
 */
function mkdir(string $directory, int $permissions = 0777, bool $recursive = false, $context = null): bool
{
}

/**
 * @param null|resource $context
 */
function rename(string $from, string $to, $context = null): bool
{
}

/**
 * @param null|resource $context
 */
function copy(string $from, string $to, $context = null): bool
{
}

function tempnam(string $directory, string $prefix): string|false
{
}

/**
 * @return resource|false
 */
function tmpfile()
{
}

/**
 * @param null|resource $context
 *
 * @return array<int, string>|false
 */
function file(string $filename, int $flags = 0, $context = null): array|false
{
}

/**
 * @param null|resource $context
 */
function file_get_contents(
    string $filename,
    bool $use_include_path = false,
    $context = null,
    int $offset = 0,
    null|int $length,
): string|false {
}

/**
 * @param null|resource $context
 *
 * @return int<0, max>|false
 */
function file_put_contents(string $filename, mixed $data, int $flags = 0, $context = null): int|false
{
}
