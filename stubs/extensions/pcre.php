<?php

const PREG_PATTERN_ORDER = 1;

const PREG_JIT_STACKLIMIT_ERROR = 6;

const PREG_SET_ORDER = 2;

const PREG_OFFSET_CAPTURE = 256;

const PREG_SPLIT_NO_EMPTY = 1;

const PREG_SPLIT_DELIM_CAPTURE = 2;

const PREG_SPLIT_OFFSET_CAPTURE = 4;

const PREG_GREP_INVERT = 1;

const PREG_NO_ERROR = 0;

const PREG_INTERNAL_ERROR = 1;

const PREG_BACKTRACK_LIMIT_ERROR = 2;

const PREG_RECURSION_LIMIT_ERROR = 3;

const PREG_BAD_UTF8_ERROR = 4;

const PREG_BAD_UTF8_OFFSET_ERROR = 5;

const PREG_UNMATCHED_AS_NULL = 512;

const PCRE_VERSION = '8.31 2012-07-06';

const PCRE_VERSION_MAJOR = 10;

const PCRE_VERSION_MINOR = 42;

const PCRE_JIT_SUPPORT = 1;

/**
 * @param-out array<string> $matches
 *
 * @return int|false
 */
function preg_match(string $pattern, string $subject, &$matches = [], int $flags = 0, int $offset = 0): int|false
{
}

/**
 * @param-out list<array<string>> $matches
 *
 * @pure
 */
function preg_match_all(string $pattern, string $subject, &$matches, int $flags = 0, int $offset = 0): int|false
{
}

/**
 * @param string|array<string> $pattern
 * @param string|array<string> $replacement <p>
 * @param string|array<string> $subject <p>
 *
 * @param-out int $count
 *
 * @return ($subject is string ? string|null : array<string>|null)
 *
 * @pure
 */
function preg_replace(
    array|string $pattern,
    array|string $replacement,
    array|string $subject,
    int $limit = -1,
    &$count = null,
): array|string|null {
}

/**
 * @param string|array<string> $pattern
 * @param (callable(array<string>): string) $callback
 * @param string|array<string> $subject
 *
 * @param-out int $count
 *
 * @return ($subject is string ? string|null : array<string>|null)
 */
function preg_replace_callback(
    array|string $pattern,
    callable $callback,
    array|string $subject,
    int $limit = -1,
    &$count = null,
    int $flags = 0,
): array|string|null {
}

/**
 * @param array<(callable(array<string>): string)> $pattern
 * @param string|array<string> $subject
 *
 * @param-out int $count
 *
 * @return ($subject is string ? string|false : array<string>|false)
 */
function preg_replace_callback_array(
    array $pattern,
    array|string $subject,
    int $limit = -1,
    &$count = null,
    int $flags = 0,
): array|string|null {
}

/**
 * @param string|array<string> $pattern
 * @param string|array<string> $replacement
 * @param string|array<string> $subject
 *
 * @param-out int $count
 *
 * @return ($subject is string ? string|null : array<string>|null)
 */
function preg_filter(
    array|string $pattern,
    array|string $replacement,
    array|string $subject,
    int $limit = -1,
    &$count = null,
): array|string|null {
}

/**
 * @return ($subject is non-empty-string ? list<non-empty-string>|false : list<string>|false)
 *
 * @pure
 */
function preg_split(string $pattern, string $subject, int $limit = -1, int $flags = 0): array|false
{
}

/**
 * @return ($str is non-empty-string ? non-empty-string : string)
 *
 * @pure
 */
function preg_quote(string $str, null|string $delimiter = null): string
{
}

/**
 * @template K of array-key
 *
 * @param array<K, string> $array
 *
 * @return array<K, string>|false
 *
 * @pure
 */
function preg_grep(string $pattern, array $array, int $flags = 0): array|false
{
}

function preg_last_error(): int
{
}

function preg_last_error_msg(): string
{
}
