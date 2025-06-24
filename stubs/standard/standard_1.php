<?php

/**
 * @pure
 */
function strtoupper(string $string): string
{
}

/**
 * @pure
 */
function strtolower(string $string): string
{
}

/**
 * @param int<0, max> $offset
 *
 * @return int<0, max>|false
 *
 * @pure
 */
function strpos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function stripos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function strrpos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return int<0, max>|false
 *
 * @pure
 */
function strripos(string $haystack, string $needle, int $offset = 0): int|false
{
}

/**
 * @return ($string is non-empty-string ? non-empty-string : string)
 *
 * @pure
 */
function strrev(string $string): string
{
}

/**
 * @pure
 */
function hebrev(string $string, int $max_chars_per_line = 0): string
{
}

/**
 * @pure
 */
function nl2br(string $string, bool $use_xhtml = true): string
{
}

/**
 * @pure
 *
 * @return ($path is non-empty-string ? non-empty-string : string)
 */
function basename(string $path, string $suffix = ''): string
{
}

/**
 * @pure
 *
 * @return ($path is non-empty-string ? non-empty-string : string)
 */
function dirname(string $path, int $levels = 1): string
{
}

/**
 * @param 1|2|4|8|15 $flags
 *
 * @return ($flags is 15 ? array{dirname?: string, basename: string, extension?: string, filename: string} : string)
 */
function pathinfo(string $path, int $flags = PATHINFO_ALL): array|string
{
}

/**
 * @pure
 */
function stripslashes(string $string): string
{
}

/**
 * @pure
 */
function stripcslashes(string $string): string
{
}

/**
 * @pure
 */
function strstr(string $haystack, string $needle, bool $before_needle = false): string|false
{
}

/**
 * @pure
 */
function stristr(string $haystack, string $needle, bool $before_needle = false): string|false
{
}

/**
 * @pure
 */
function strrchr(string $haystack, string $needle, bool $before_needle = false): string|false
{
}

/**
 * @template T as string
 *
 * @param T $string
 *
 * @return (T is non-empty-string ? non-empty-string : string)
 *
 * @pure
 */
function str_shuffle(string $string): string
{
}

/**
 * @template T as 0|1|2
 *
 * @param T $format
 *
 * @return (T is 0 ? int<0, max> : (
 *   T is 1 ? list<non-empty-string> : (
 *     T is 2 ? array<int<0, max>, non-empty-string> : (
 *       int<0, max>|list<non-empty-string>|array<int<0, max>, non-empty-string>
 *     )
 *   )
 * ))
 *
 * @pure
 */
function str_word_count(string $string, int $format = 0, null|string $characters): array|int
{
}

/**
 * @param int<1, max> $length
 *
 * @return list<string>
 *
 * @pure
 */
function str_split(string $string, int $length = 1): array
{
}

/**
 * @pure
 */
function strpbrk(string $string, string $characters): string|false
{
}

/**
 * @pure
 */
function substr_compare(
    string $haystack,
    string $needle,
    int $offset,
    null|int $length,
    bool $case_insensitive = false,
): int {
}

/**
 * @pure
 */
function strcoll(string $string1, string $string2): int
{
}

/**
 * @param string $string
 * @param null|int<0, max> $length
 *
 * @pure
 */
function substr(string $string, int $offset, null|int $length = null): string
{
}

/**
 * @template K as array-key
 *
 * @param string|array<K, string> $string
 * @param string|array<string> $replace
 * @param int|array<int> $offset
 * @param null|int<0, max>|array<int<0, max>> $length
 *
 * @return ($string is string ? string : array<K, string>)
 *
 * @pure
 */
function substr_replace(
    array|string $string,
    array|string $replace,
    array|int $offset,
    array|int|null $length = null,
): array|string {
}

/**
 * @pure
 */
function quotemeta(string $string): string
{
}

/**
 * @pure
 */
function ucfirst(string $string): string
{
}

/**
 * @pure
 */
function lcfirst(string $string): string
{
}

/**
 * @pure
 */
function ucwords(string $string, string $separators = " \t\r\n\f\v"): string
{
}

/**
 * @pure
 */
function strtr(string $string, string $from, string $to): string
{
}

/**
 * @pure
 */
function strtr(string $str, array $replace_pairs): string
{
}

/**
 * @pure
 */
function addslashes(string $string): string
{
}

/**
 * @pure
 */
function addcslashes(string $string, string $characters): string
{
}

/**
 * @pure
 */
function rtrim(string $string, string $characters = " \n\r\t\v\0"): string
{
}

/**
 * @param string|array<string> $search
 * @param string|array<string> $replace
 * @param string|array<string> $subject
 *
 * @return ($subject is string ? string : (
 *   $subject is array<string> ? array<string> : string|array<string>
 * ))
 *
 * @pure
 */
function str_replace(
    array|string $search,
    array|string $replace,
    array|string $subject,
    null|int &$count = null,
): array|string {
}

/**
 * @param string|array<string> $search
 * @param string|array<string> $replace
 * @param string|array<string> $subject
 *
 * @return ($subject is string ? string : (
 *   $subject is array<string> ? array<string> : string|array<string>
 * ))
 *
 * @pure
 */
function str_ireplace(
    array|string $search,
    array|string $replace,
    array|string $subject,
    null|int &$count = null,
): array|string {
}

/**
 * @pure
 */
function str_repeat(string $string, int $times): string
{
}

/**
 * @pure
 */
function count_chars(string $string, int $mode = 0): array|string
{
}

/**
 * @pure
 */
function chunk_split(string $string, int $length = 76, string $separator = "\r\n"): string
{
}

/**
 * @pure
 */
function trim(string $string, string $characters = " \n\r\t\v\0"): string
{
}

/**
 * @pure
 */
function ltrim(string $string, string $characters = " \n\r\t\v\0"): string
{
}

/**
 * @param array<string>|string|null $allowed_tags
 *
 * @pure
 */
function strip_tags(string $string, string|array|null $allowed_tags = null): string
{
}

/**
 * @pure
 */
function similar_text(string $string1, string $string2, &$percent): int
{
}

/**
 * @return list<non-empty-string>
 *
 * @pure
 */
function explode(string $separator, string $string, int $limit = PHP_INT_MAX): array
{
}

/**
 * @param array<string>|string $separator
 * @param array<string>|null $array
 *
 * @pure
 */
function implode(array|string $separator = '', null|array $array = null): string
{
}

/**
 * @param array<string>|string $separator
 * @param array<string>|null $array
 *
 * @pure
 */
function join(array|string $separator = '', null|array $array = null): string
{
}

/**
 * @param string|array<string>|int $locales
 * @param string|array<string> ...$rest
 */
function setlocale(int $category, string|int|array $locales, string|array ...$rest): string|false
{
}

/**
 * @return array{
 *   decimal_point: string,
 *   thousands_sep: string,
 *   grouping: array<int, int>,
 *   int_curr_symbol: string,
 *   currency_symbol: string,
 *   mon_decimal_point: string,
 *   mon_thousands_sep: string,
 *   mon_grouping: string,
 *   positive_sign: string,
 *   negative_sign: string,
 *   int_frac_digits: string,
 *   frac_digits: string,
 *   p_cs_precedes: bool,
 *   p_sep_by_space: bool,
 *   n_cs_precedes: bool,
 *   n_sep_by_space: bool,
 *   p_sign_posn: int,
 *   n_sign_posn: int
 * }
 *
 * @pure
 */
function localeconv(): array
{
}
