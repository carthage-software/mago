<?php

use JetBrains\PhpStorm\ArrayShape;
use JetBrains\PhpStorm\Deprecated;
use JetBrains\PhpStorm\ExpectedValues;
use JetBrains\PhpStorm\Internal\LanguageLevelTypeAware;
use JetBrains\PhpStorm\Internal\PhpStormStubsElementAvailable;
use JetBrains\PhpStorm\Pure;

function syslog(int $priority, string $message): true
{
}

function closelog(): true
{
}

/**
 * @param (callable(): void) $callback
 */
function header_register_callback(callable $callback): bool
{
}

/**
 * @return false|array{
 *  0: int,
 *  1: int,
 *  2: int,
 *  3: string,
 *  bits: int,
 *  channels: int,
 *  mime: string
 * }
 */
function getimagesizefromstring(string $string, &$image_info): array|false
{
}

/**
 * @param resource $stream
 */
function stream_set_chunk_size($stream, int $size): int
{
}

/**
 * @pure
 */
function metaphone(string $string, int $max_phonemes = 0): string
{
}

/**
 * @param (callable(string, null|int): string)|null $callback
 */
function ob_start(callable|null $callback = null, int $chunk_size = 0, int $flags = PHP_OUTPUT_HANDLER_STDFLAGS): bool
{
}

function ob_flush(): bool
{
}

function ob_clean(): bool
{
}

function ob_end_flush(): bool
{
}

function ob_end_clean(): bool
{
}

function ob_get_flush(): string|false
{
}

function ob_get_clean(): string|false
{
}

function ob_get_length(): int|false
{
}

function ob_get_level(): int
{
}

/**
 * @return array{
 *   level: int,
 *   type: int,
 *   flags: int,
 *   name: string,
 *   del: int,
 *   chunk_size: int,
 *   buffer_size: int,
 *   buffer_used: int
 * }
 */
function ob_get_status(bool $full_status = false): array
{
}

function ob_get_contents(): string|false
{
}

function ob_implicit_flush(bool $enable = true): void
{
}

/**
 * @return list<non-empty-string>
 */
function ob_list_handlers(): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function ksort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function krsort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function natsort(array &$array): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function natcasesort(array &$array): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function asort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @param-out array<K, V> $array
 */
function arsort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template T
 *
 * @param array<T> $array
 *
 * @param-out list<T> $array
 */
function sort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template T
 *
 * @param array<T> $array
 *
 * @param-out list<T> $array
 */
function rsort(array &$array, int $flags = SORT_REGULAR): true
{
}

/**
 * @template T
 *
 * @param array<T> $array
 * @param (callable(T, T): int) $callback
 *
 * @param-out list<T> $array
 */
function usort(array &$array, callable $callback): true
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(V, V): int) $callback
 *
 * @param-out array<K, V> $array
 */
function uasort(array &$array, callable $callback): true
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param (callable(K, K): int) $callback
 *
 * @param-out array<K, V> $array
 */
function uksort(array &$array, callable $callback): true
{
}

/**
 * @template T
 *
 * @param array<T> $array
 *
 * @param-out list<T> $array
 */
function shuffle(array &$array): true
{
}

/**
 * Apply a user function to every member of an array
 * @link https://php.net/manual/en/function.array-walk.php
 * @param array|object &$array <p>
 * The input array.
 * </p>
 * @param callable $callback <p>
 * Typically, funcname takes on two parameters.
 * The array parameter's value being the first, and
 * the key/index second.
 * </p>
 * <p>
 * If funcname needs to be working with the
 * actual values of the array, specify the first parameter of
 * funcname as a
 * reference. Then,
 * any changes made to those elements will be made in the
 * original array itself.
 * </p>
 * <p>
 * Users may not change the array itself from the
 * callback function. e.g. Add/delete elements, unset elements, etc. If
 * the array that array_walk is applied to is
 * changed, the behavior of this function is undefined, and unpredictable.
 * </p>
 * @param mixed $arg [optional] <p>
 * If the optional userdata parameter is supplied,
 * it will be passed as the third parameter to the callback
 * funcname.
 * </p>
 */
#[LanguageLevelTypeAware(['8.2' => 'true'], default: 'bool')]
function array_walk(object|array &$array, callable $callback, mixed $arg)
{
}

/**
 * Apply a user function recursively to every member of an array
 * @link https://php.net/manual/en/function.array-walk-recursive.php
 * @param array|object &$array <p>
 * The input array.
 * </p>
 * @param callable $callback <p>
 * Typically, funcname takes on two parameters.
 * The input parameter's value being the first, and
 * the key/index second.
 * </p>
 * <p>
 * If funcname needs to be working with the
 * actual values of the array, specify the first parameter of
 * funcname as a
 * reference. Then,
 * any changes made to those elements will be made in the
 * original array itself.
 * </p>
 * @param mixed $arg [optional] <p>
 * If the optional userdata parameter is supplied,
 * it will be passed as the third parameter to the callback
 * funcname.
 * </p>
 */
#[LanguageLevelTypeAware(['8.2' => 'true'], default: 'bool')]
function array_walk_recursive(object|array &$array, callable $callback, mixed $arg)
{
}

/**
 * @return int<0, max>
 *
 * @pure
 */
function count(Countable|array $value, int $mode = COUNT_NORMAL): int
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|false
 */
function end(object|array &$array): mixed
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|false
 */
function prev(object|array &$array): mixed
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|false
 */
function next(object|array &$array): mixed
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|false
 */
function reset(object|array &$array): mixed
{
}

/**
 * @template T
 *
 * @param object|array<T> $array
 *
 * @return T|null
 *
 * @pure
 */
function current(object|array $array): mixed
{
}

/**
 * @template K of array-key
 *
 * @param object|array<K, mixed> $array
 *
 * @return K|null
 *
 * @pure
 */
function key(object|array $array): string|int|null
{
}

/**
 * @template T
 *
 * @param array<T>|T $value
 * @param T ...$values
 *
 * @return T
 *
 * @pure
 */
function min(mixed $value, mixed ...$values): mixed
{
}

/**
 * @template T
 *
 * @param array<T>|T $value
 * @param T ...$values
 *
 * @return T
 *
 * @pure
 */
function max(mixed $value, mixed ...$values): mixed
{
}

/**
 * Checks if a value exists in an array
 * @link https://php.net/manual/en/function.in-array.php
 * @param mixed $needle <p>
 * The searched value.
 * </p>
 * <p>
 * If needle is a string, the comparison is done
 * in a case-sensitive manner.
 * </p>
 * @param array $haystack <p>
 * The array.
 * </p>
 * @param bool $strict [optional] <p>
 * If the third parameter strict is set to true
 * then the in_array function will also check the
 * types of the
 * needle in the haystack.
 * </p>
 * @return bool true if needle is found in the array,
 * false otherwise.
 */
#[Pure]
function in_array(mixed $needle, array $haystack, bool $strict = false): bool
{
}

/**
 * Searches the array for a given value and returns the first corresponding key if successful
 * @link https://php.net/manual/en/function.array-search.php
 * @param mixed $needle <p>
 * The searched value.
 * </p>
 * <p>
 * If needle is a string, the comparison is done
 * in a case-sensitive manner.
 * </p>
 * @param array $haystack <p>
 * The array.
 * </p>
 * @param bool $strict [optional] <p>
 * If the third parameter strict is set to true
 * then the array_search function will also check the
 * types of the
 * needle in the haystack.
 * </p>
 * @return int|string|false the key for needle if it is found in the
 * array, false otherwise.
 * </p>
 * <p>
 * If needle is found in haystack
 * more than once, the first matching key is returned. To return the keys for
 * all matching values, use array_keys with the optional
 * search_value parameter instead.
 */
#[Pure]
function array_search(mixed $needle, array $haystack, bool $strict = false): string|int|false
{
}

/**
 * Import variables into the current symbol table from an array
 * @link https://php.net/manual/en/function.extract.php
 * @param array &$array <p>
 * Note that prefix is only required if
 * extract_type is EXTR_PREFIX_SAME,
 * EXTR_PREFIX_ALL, EXTR_PREFIX_INVALID
 * or EXTR_PREFIX_IF_EXISTS. If
 * the prefixed result is not a valid variable name, it is not
 * imported into the symbol table. Prefixes are automatically separated from
 * the array key by an underscore character.
 * </p>
 * @param int $flags <p>
 * The way invalid/numeric keys and collisions are treated is determined
 * by the extract_type. It can be one of the
 * following values:
 * EXTR_OVERWRITE
 * If there is a collision, overwrite the existing variable.</p>
 * @param string $prefix <p>Only overwrite the variable if it already exists in the
 * current symbol table, otherwise do nothing. This is useful
 * for defining a list of valid variables and then extracting
 * only those variables you have defined out of
 * $_REQUEST, for example.</p>
 * @return int the number of variables successfully imported into the symbol
 * table.
 */
function extract(
    array &$array,
    #[ExpectedValues(flags: [
        EXTR_OVERWRITE,
        EXTR_SKIP,
        EXTR_PREFIX_SAME,
        EXTR_PREFIX_ALL,
        EXTR_PREFIX_INVALID,
        EXTR_IF_EXISTS,
        EXTR_PREFIX_IF_EXISTS,
        EXTR_REFS,
    ])]
    int $flags = EXTR_OVERWRITE,
    string $prefix = '',
): int {
}

/**
 * Create array containing variables and their values
 * @link https://php.net/manual/en/function.compact.php
 * @param mixed $var_name <p>
 * compact takes a variable number of parameters.
 * Each parameter can be either a string containing the name of the
 * variable, or an array of variable names. The array can contain other
 * arrays of variable names inside it; compact
 * handles it recursively.
 * </p>
 * @param mixed ...$var_names
 * @return array the output array with all the variables added to it.
 *
 * @pure
 */
function compact($var_name, ...$var_names): array
{
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return (
 *   $start_index is 0 ?
 *   ($count is int<1, max> ? non-empty-list<T> : list<T>) :
 *   ($count is int<1, max> ? non-empty-array<int, T> : array<int, T>)
 * )
 *
 * @pure
 */
function array_fill(int $start_index, int $count, mixed $value): array
{
}

/**
 * Fill an array with values, specifying keys
 * @link https://php.net/manual/en/function.array-fill-keys.php
 * @param array $keys <p>
 * Array of values that will be used as keys. Illegal values
 * for key will be converted to string.
 * </p>
 * @param mixed $value <p>
 * Value to use for filling
 * </p>
 * @return array the filled array
 */
#[Pure]
function array_fill_keys(array $keys, mixed $value): array
{
}

/**
 * Create an array containing a range of elements
 * @link https://php.net/manual/en/function.range.php
 * @param mixed $start <p>
 * First value of the sequence.
 * </p>
 * @param mixed $end <p>
 * The sequence is ended upon reaching the end value.
 * </p>
 * @param positive-int|float $step [optional] <p>
 * If a step value is given, it will be used as the
 * increment between elements in the sequence. step
 * should be given as a positive number. If not specified,
 * step will default to 1.
 * </p>
 * @return array an array of elements from start to
 * end, inclusive.
 */
#[Pure]
function range(string|int|float $start, string|int|float $end, int|float $step = 1): array
{
}

/**
 * Sort multiple or multi-dimensional arrays
 * @link https://php.net/manual/en/function.array-multisort.php
 * @param array &$array <p>
 * An array being sorted.
 * </p>
 * @param  &...$rest [optional] <p>
 * More arrays, optionally followed by sort order and flags.
 * Only elements corresponding to equivalent elements in previous arrays are compared.
 * In other words, the sort is lexicographical.
 * </p>
 * @return bool true on success or false on failure.
 */
function array_multisort(&$array, $sort_order = SORT_ASC, $sort_flags = SORT_REGULAR, &...$rest): bool
{
}

/**
 * Push elements onto the end of array
 * Since 7.3.0 this function can be called with only one parameter.
 * For earlier versions at least two parameters are required.
 * @link https://php.net/manual/en/function.array-push.php
 * @param array &$array <p>
 * The input array.
 * </p>
 * @param mixed ...$values <p>
 * The pushed variables.
 * </p>
 * @return int the number of elements in the array.
 */
function array_push(array &$array, mixed ...$values): int
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param-out ($array is list ? list<V> : array<K, V>) $array
 *
 * @return V|null
 */
function array_pop(array &$array): mixed
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param-out ($array is list ? list<V> : array<K, V>) $array
 *
 * @return V|null
 *
 * @pure
 */
function array_shift(array &$array): mixed
{
}

/**
 * Prepend elements to the beginning of an array
 * Since 7.3.0 this function can be called with only one parameter.
 * For earlier versions at least two parameters are required.
 * @link https://php.net/manual/en/function.array-unshift.php
 * @param array &$array <p>
 * The input array.
 * </p>
 * @param mixed ...$values <p>
 * The prepended variables.
 * </p>
 * @return int the number of elements in the array.
 */
function array_unshift(array &$array, mixed ...$values): int
{
}

/**
 * Remove a portion of the array and replace it with something else
 * @link https://php.net/manual/en/function.array-splice.php
 * @param array &$array <p>
 * The input array.
 * </p>
 * @param int $offset <p>
 * If offset is positive then the start of removed
 * portion is at that offset from the beginning of the
 * input array. If offset
 * is negative then it starts that far from the end of the
 * input array.
 * </p>
 * @param int|null $length [optional] <p>
 * If length is omitted, removes everything
 * from offset to the end of the array. If
 * length is specified and is positive, then
 * that many elements will be removed. If
 * length is specified and is negative then
 * the end of the removed portion will be that many elements from
 * the end of the array. Tip: to remove everything from
 * offset to the end of the array when
 * replacement is also specified, use
 * count($input) for
 * length.
 * </p>
 * @param mixed $replacement <p>
 * If replacement array is specified, then the
 * removed elements are replaced with elements from this array.
 * </p>
 * <p>
 * If offset and length
 * are such that nothing is removed, then the elements from the
 * replacement array are inserted in the place
 * specified by the offset. Note that keys in
 * replacement array are not preserved.
 * </p>
 * <p>
 * If replacement is just one element it is
 * not necessary to put array()
 * around it, unless the element is an array itself.
 * </p>
 * @return array the array consisting of the extracted elements.
 */
function array_splice(array &$array, int $offset, null|int $length, mixed $replacement = []): array
{
}

/**
 * @template K of array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @return ($preserve_keys is true ? array<K, V> : list<V>) $array
 *
 * @pure
 */
function array_slice(array $array, int $offset, null|int $length = null, bool $preserve_keys = false): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> ...$arrays
 *
 * @return array<K, V>
 *
 * @pure
 */
function array_merge(array ...$arrays): array
{
}
