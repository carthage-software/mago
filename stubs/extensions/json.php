<?php

interface JsonSerializable
{
    public function jsonSerialize(): mixed;
}

class JsonException extends Exception
{
}

function json_encode(mixed $value, int $flags = 0, int $depth = 512): string|false
{
}

function json_decode(string $json, null|bool $associative = null, int $depth = 512, int $flags = 0): mixed
{
}

/**
 * @pure
 */
function json_last_error(): int
{
}

/**
 * @pure
 */
function json_last_error_msg(): string
{
}

/**
 * @pure
 */
function json_validate(string $json, int $depth = 512, int $flags = 0): bool
{
}

const JSON_HEX_TAG = 1;

const JSON_HEX_AMP = 2;

const JSON_HEX_APOS = 4;

const JSON_HEX_QUOT = 8;

const JSON_FORCE_OBJECT = 16;

const JSON_NUMERIC_CHECK = 32;

const JSON_UNESCAPED_SLASHES = 64;

const JSON_PRETTY_PRINT = 128;

const JSON_UNESCAPED_UNICODE = 256;

const JSON_PARTIAL_OUTPUT_ON_ERROR = 512;

const JSON_ERROR_STATE_MISMATCH = 2;

const JSON_ERROR_CTRL_CHAR = 3;

const JSON_ERROR_UTF8 = 5;

const JSON_ERROR_RECURSION = 6;

const JSON_ERROR_INF_OR_NAN = 7;

const JSON_ERROR_UNSUPPORTED_TYPE = 8;

const JSON_ERROR_NONE = 0;

const JSON_ERROR_DEPTH = 1;

const JSON_ERROR_SYNTAX = 4;

const JSON_OBJECT_AS_ARRAY = 1;

const JSON_PARSER_NOTSTRICT = 4;

const JSON_BIGINT_AS_STRING = 2;

const JSON_PRESERVE_ZERO_FRACTION = 1024;

const JSON_UNESCAPED_LINE_TERMINATORS = 2048;

const JSON_INVALID_UTF8_IGNORE = 1048576;

const JSON_INVALID_UTF8_SUBSTITUTE = 2097152;

const JSON_ERROR_INVALID_PROPERTY_NAME = 9;

const JSON_ERROR_UTF16 = 10;

const JSON_THROW_ON_ERROR = 4194304;

const JSON_ERROR_NON_BACKED_ENUM = 11;
