<?php

/**
 * @pure
 */
function iconv(string $from_encoding, string $to_encoding, string $string): string|false {}

/**
 * @pure
 */
function ob_iconv_handler(string $contents, int $status): string {}

/**
 * @param 'all'|'input_encoding'|'output_encoding'|'internal_encoding' $type
 *
 * @return false|string|array{input_encoding: string, output_encoding: string, internal_encoding: string}
 *
 * @pure
 */
function iconv_get_encoding(string $type = 'all'): array|string|false {}

function iconv_set_encoding(string $type, string $encoding): bool {}

/**
 * @return false|int<0, max>
 *
 * @pure
 */
function iconv_strlen(string $string, ?string $encoding = null): int|false {}

/**
 * @pure
 */
function iconv_substr(string $string, int $offset, ?int $length, ?string $encoding = null): string|false {}

/**
 * @pure
 */
function iconv_strpos(string $haystack, string $needle, int $offset = 0, ?string $encoding = null): int|false {}

/**
 * @pure
 */
function iconv_strrpos(string $haystack, string $needle, ?string $encoding = null): int|false {}

/**
 * @param array{'scheme'?: string, 'input-charset'?: string, 'output-charset'?: string, 'line-length'?: int, 'line-break-chars'?: string} $options
 *
 * @pure
 */
function iconv_mime_encode(string $field_name, string $field_value, array $options = []): string|false {}

/**
 * @param 1|2 $mode
 *
 * @pure
 */
function iconv_mime_decode(string $string, int $mode = 0, ?string $encoding = null): string|false {}

/**
 * @param 1|2 $mode
 *
 * @return array<string, string|non-empty-list<string>>|false
 *
 * @pure
 */
function iconv_mime_decode_headers(string $headers, int $mode = 0, ?string $encoding = null): array|false {}

const ICONV_IMPL = 'libiconv';

const ICONV_VERSION = 2.17;

const ICONV_MIME_DECODE_STRICT = 1;

const ICONV_MIME_DECODE_CONTINUE_ON_ERROR = 2;
