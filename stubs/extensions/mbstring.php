<?php

const MB_CASE_UPPER = 0;

const MB_CASE_LOWER = 1;

const MB_CASE_TITLE = 2;

const MB_CASE_FOLD = 3;

const MB_CASE_UPPER_SIMPLE = 4;

const MB_CASE_LOWER_SIMPLE = 5;

const MB_CASE_TITLE_SIMPLE = 6;

const MB_CASE_FOLD_SIMPLE = 7;

const MB_ONIGURUMA_VERSION = '6.9.9';

/**
 * @pure
 */
function mb_convert_case(string $string, int $mode, null|string $encoding): string
{
}

/**
 * @pure
 */
function mb_strtoupper(string $string, null|string $encoding): string
{
}

/**
 * @pure
 */
function mb_strtolower(string $string, null|string $encoding): string
{
}

function mb_language(null|string $language): string|bool
{
}

function mb_internal_encoding(null|string $encoding): string|bool
{
}

/**
 * @param 'G'|'P'|'C'|'S'|'L'|'I'|null $type
 *
 * @return ($type is 'G'|'P'|'C'|'S'|'L'|null ? string|false : list<string>|false)
 */
function mb_http_input(null|string $type): array|string|false
{
}

function mb_http_output(null|string $encoding): string|bool
{
}

/**
 * @return bool|list<string>
 */
function mb_detect_order(array|string|null $encoding = null): array|true
{
}

function mb_substitute_character(string|int|null $substitute_character = null): string|int|bool
{
}

function mb_parse_str(string $string, array &$result): bool
{
}

/**
 * @pure
 */
function mb_output_handler(string $string, int $status): string
{
}

/**
 * @pure
 */
function mb_preferred_mime_name(string $encoding): string|false
{
}

/**
 * @return int<0, max>
 *
 * @pure
 */
function mb_strlen(string $string, null|string $encoding = null): int
{
}

/**
 * @return int<0,max>|false
 *
 * @pure
 */
function mb_strpos(string $haystack, string $needle, int $offset = 0, null|string $encoding): int|false
{
}

/**
 * @return int<0,max>|false
 *
 * @pure
 */
function mb_strrpos(string $haystack, string $needle, int $offset = 0, null|string $encoding): int|false
{
}

/**
 * @return int<0,max>|false
 *
 * @pure
 */
function mb_stripos(string $haystack, string $needle, int $offset = 0, null|string $encoding): int|false
{
}

/**
 * @return int<0,max>|false
 *
 * @pure
 */
function mb_strripos(string $haystack, string $needle, int $offset = 0, null|string $encoding): int|false
{
}

/**
 * @pure
 */
function mb_strstr(string $haystack, string $needle, bool $before_needle = false, null|string $encoding): string|false
{
}

/**
 * @pure
 */
function mb_strrchr(string $haystack, string $needle, bool $before_needle = false, null|string $encoding): string|false
{
}

/**
 * @pure
 */
function mb_stristr(string $haystack, string $needle, bool $before_needle = false, null|string $encoding): string|false
{
}

/**
 * @pure
 */
function mb_strrichr(string $haystack, string $needle, bool $before_needle = false, null|string $encoding): string|false
{
}

/**
 * @pure
 */
function mb_substr_count(string $haystack, string $needle, null|string $encoding): int
{
}

/**
 * @pure
 */
function mb_substr(string $string, int $start, null|int $length, null|string $encoding): string
{
}

/**
 * @pure
 */
function mb_strcut(string $string, int $start, null|int $length, null|string $encoding): string
{
}

/**
 * @pure
 */
function mb_strwidth(string $string, null|string $encoding): int
{
}

/**
 * @pure
 */
function mb_strimwidth(string $string, int $start, int $width, string $trim_marker = '', null|string $encoding): string
{
}

/**
 * @param string|array<string> $string
 * @param string|array<string>|null $from_encoding
 *
 * @return ($string is string ? string|false : array|false)
 *
 * @pure
 */
function mb_convert_encoding(
    array|string $string,
    string $to_encoding,
    array|string|null $from_encoding = null,
): array|string|false {
}

/**
 * @param string|array<string>|null $encodings
 *
 * @pure
 */
function mb_detect_encoding(string $string, array|string|null $encodings = null, bool $strict = false): string|false
{
}

/**
 * @return list<string>
 *
 * @pure
 */
function mb_list_encodings(): array
{
}

/**
 * @return list<string>
 *
 * @pure
 */
function mb_encoding_aliases(string $encoding): array
{
}

/**
 * @pure
 */
function mb_convert_kana(string $string, string $mode = 'KV', null|string $encoding): string
{
}

/**
 * @pure
 */
function mb_encode_mimeheader(
    string $string,
    null|string $charset,
    null|string $transfer_encoding,
    string $newline = "\r\n",
    int $indent = 0,
): string {
}

/**
 * @pure
 */
function mb_decode_mimeheader(string $string): string
{
}

/**
 * @param string|string[] $from_encoding
 */
function mb_convert_variables(
    string $to_encoding,
    array|string $from_encoding,
    string|array|object &$var,
    string|array|object &...$vars,
): string|false {
}

/**
 * @param int[] $map <p>
 *
 * @pure
 */
function mb_encode_numericentity(string $string, array $map, null|string $encoding = null, bool $hex = false): string
{
}

/**
 * @param int[] $map
 *
 * @pure
 */
function mb_decode_numericentity(string $string, array $map, null|string $encoding = null): string
{
}

function mb_send_mail(
    string $to,
    string $subject,
    string $message,
    array|string $additional_headers = [],
    null|string $additional_params,
): bool {
}

/**
 * @return string|int|false|null|array{
 *     internal_encoding: string,
 *     http_input: string,
 *     http_output: string,
 *     http_output_conv_mimetypes: string,
 *     mail_charset: string,
 *     mail_header_encoding: string,
 *     mail_body_encoding: string,
 *     illegal_chars: string,
 *     encoding_translation: string,
 *     language: string,
 *     detect_order: string,
 *     substitute_character: string,
 *     strict_detection: string
 * }
 *
 * @pure
 */
function mb_get_info(string $type = 'all'): array|string|int|false|null
{
}

/**
 * @param string|string[]|null $value
 *
 * @pure
 */
function mb_check_encoding(array|string|null $value = null, null|string $encoding): bool
{
}

function mb_regex_encoding(null|string $encoding): string|bool
{
}

function mb_regex_set_options(null|string $options): string
{
}

/**
 * @param string[] &$matches
 */
function mb_ereg(string $pattern, string $string, &$matches): bool
{
}

/**
 * @param string[] &$matches
 */
function mb_eregi(string $pattern, string $string, &$matches): bool
{
}

/**
 * @pure
 */
function mb_ereg_replace(
    string $pattern,
    string $replacement,
    string $string,
    null|string $options = null,
): string|false|null {
}

function mb_ereg_replace_callback(
    string $pattern,
    callable $callback,
    string $string,
    null|string $options = null,
): string|false|null {
}

/**
 * @pure
 */
function mb_eregi_replace(
    string $pattern,
    string $replacement,
    string $string,
    null|string $options = null,
): string|false|null {
}

/**
 * @return string[]|false
 * @pure
 */
function mb_split(string $pattern, string $string, int $limit = -1): array|false
{
}

/**
 * @pure
 */
function mb_ereg_match(string $pattern, string $string, null|string $options): bool
{
}

/**
 * @pure
 */
function mb_ereg_search(null|string $pattern, null|string $options): bool
{
}

/**
 * @return list<int>|false
 *
 * @pure
 */
function mb_ereg_search_pos(null|string $pattern, null|string $options): array|false
{
}

/**
 * @return list<string>|false
 *
 * @pure
 */
function mb_ereg_search_regs(null|string $pattern, null|string $options): array|false
{
}

function mb_ereg_search_init(string $string, null|string $pattern, null|string $options): bool
{
}

/**
 * @return list<string>|false
 *
 * @pure
 */
function mb_ereg_search_getregs(): array|false
{
}

/**
 * @pure
 */
function mb_ereg_search_getpos(): int
{
}

/**
 * @pure
 */
function mb_ereg_search_setpos(int $offset): bool
{
}

/**
 * @pure
 */
function mb_chr(int $codepoint, null|string $encoding): string|false
{
}

/**
 * @return int<0, max>
 *
 * @pure
 */
function mb_ord(string $string, null|string $encoding): int|false
{
}

/**
 * @pure
 * @deprecated
 */
function mb_scrub(string $string, null|string $encoding): string
{
}

/**
 * @pure
 * @deprecated
 */
function mbereg_search_setpos($position)
{
}

/**
 * @return ($string is non-empty-string ? list<non-empty-string> : list<string>)
 *
 * @pure
 */
function mb_str_split(string $string, int $length = 1, null|string $encoding): array
{
}

function mb_str_pad(
    string $string,
    int $length,
    string $pad_string = ' ',
    int $pad_type = STR_PAD_RIGHT,
    null|string $encoding = null,
): string {
}

function mb_ucfirst(string $string, null|string $encoding = null): string
{
}

function mb_lcfirst(string $string, null|string $encoding = null): string
{
}

function mb_trim(string $string, null|string $characters = null, null|string $encoding = null): string
{
}

function mb_ltrim(string $string, null|string $characters = null, null|string $encoding = null): string
{
}

function mb_rtrim(string $string, null|string $characters = null, null|string $encoding = null): string
{
}
