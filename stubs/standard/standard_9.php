<?php

const ARRAY_FILTER_USE_BOTH = 1;

const ARRAY_FILTER_USE_KEY = 2;

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> ...$arrays
 *
 * @return array<K, V>
 *
 * @no-named-arguments
 * @pure
 */
function array_merge_recursive(array ...$arrays)
{
}

/**
 * @param array<array-key, mixed> $array
 * @param array<array-key, mixed> ...$replacements
 *
 * @return array<array-key, mixed>
 *
 * @no-named-arguments
 * @pure
 */
function array_replace(array $array, array ...$replacements): array
{
}

/**
 * @param array<array-key, mixed> $array
 * @param array<array-key, mixed> ...$replacements
 *
 * @return array<array-key, mixed>
 *
 * @no-named-arguments
 * @pure
 */
function array_replace_recursive(array $array, array ...$replacements): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param V $filter_value
 * @param bool $strict
 *
 * @return list<K>
 *
 * @no-named-arguments
 * @pure
 */
function array_keys(array $array, mixed $filter_value = null, bool $strict = false): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 *
 * @return list<V>
 *
 * @no-named-arguments
 * @pure
 */
function array_values(array $array): array
{
}

/**
 * @template K as array-key
 * @template V as array-key
 *
 * @param array<K, V> $array
 *
 * @return array<V, int>
 *
 * @no-named-arguments
 * @pure
 */
function array_count_values(array $array): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<array-key, array<K, V>> $array
 * @param K|null $column_key
 * @param K|null $index_key
 *
 * @return array<array-key, V>
 *
 * @no-named-arguments
 * @pure
 */
function array_column(array $array, string|int|null $column_key, string|int|null $index_key = null): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param bool $preserve_keys
 *
 * @return ($preserve_keys ? array<K, V> : list<V>)
 *
 * @no-named-arguments
 * @pure
 */
function array_reverse(array $array, bool $preserve_keys = false): array
{
}

/**
 * Iteratively reduce the array to a single value using a callback function
 * @link https://php.net/manual/en/function.array-reduce.php
 * @param array $array <p>
 * The input array.
 * </p>
 * @param callable $callback <p>
 * The callback function. Signature is <pre>callback ( mixed $carry , mixed $item ) : mixed</pre>
 * <blockquote>mixed <var>$carry</var> <p>The return value of the previous iteration; on the first iteration it holds the value of <var>$initial</var>.</p></blockquote>
 * <blockquote>mixed <var>$item</var> <p>Holds the current iteration value of the <var>$input</var></p></blockquote>
 * </p>
 * @param mixed $initial [optional] <p>
 * If the optional initial is available, it will
 * be used at the beginning of the process, or as a final result in case
 * the array is empty.
 * </p>
 * @return mixed the resulting value.
 * <p>
 * If the array is empty and initial is not passed,
 * array_reduce returns null.
 * </p>
 * <br/>
 * <p>
 * Example use:
 * <blockquote><pre>array_reduce(['2', '3', '4'], function($ax, $dx) { return $ax . ", {$dx}"; }, '1')  // Returns '1, 2, 3, 4'</pre></blockquote>
 * <blockquote><pre>array_reduce(['2', '3', '4'], function($ax, $dx) { return $ax + (int)$dx; }, 1)  // Returns 10</pre></blockquote>
 * <br/>
 * </p>
 * @meta
 */
function array_reduce(array $array, callable $callback, mixed $initial = null): mixed
{
}

/**
 * Pad array to the specified length with a value
 * @link https://php.net/manual/en/function.array-pad.php
 * @param array $array <p>
 * Initial array of values to pad.
 * </p>
 * @param int $length <p>
 * New size of the array.
 * </p>
 * @param mixed $value <p>
 * Value to pad if input is less than
 * pad_size.
 * </p>
 * @return array a copy of the input padded to size specified
 * by pad_size with value
 * pad_value. If pad_size is
 * positive then the array is padded on the right, if it's negative then
 * on the left. If the absolute value of pad_size is less than or equal to
 * the length of the input then no padding takes place.
 */
#[Pure]
function array_pad(array $array, int $length, mixed $value): array
{
}

/**
 * Exchanges all keys with their associated values in an array
 * @link https://php.net/manual/en/function.array-flip.php
 * @param int[]|string[] $array <p>
 * An array of key/value pairs to be flipped.
 * </p>
 * @return int[]|string[] Returns the flipped array.
 */
#[Pure]
function array_flip(array $array): array
{
}

/**
 * Changes the case of all keys in an array
 * @link https://php.net/manual/en/function.array-change-key-case.php
 * @param array $array <p>
 * The array to work on
 * </p>
 * @param int $case <p>
 * Either CASE_UPPER or
 * CASE_LOWER (default)
 * </p>
 * @return array an array with its keys lower or uppercased
 * @meta
 */
#[Pure]
function array_change_key_case(array $array, int $case = CASE_LOWER): array
{
}

/**
 * Pick one or more random keys out of an array
 * @link https://php.net/manual/en/function.array-rand.php
 * @param array $array <p>
 * The input array.
 * </p>
 * @param int $num [optional] <p>
 * Specifies how many entries you want to pick.
 * </p>
 * @return int|string|array If you are picking only one entry, array_rand
 * returns the key for a random entry. Otherwise, it returns an array
 * of keys for the random entries. This is done so that you can pick
 * random keys as well as values out of the array.
 */
function array_rand(array $array, int $num = 1): array|string|int
{
}

/**
 * @template K
 * @template V
 *
 * @param array<K, V> $array
 * @param int<0, 5> $flags
 *
 * @return array<K, V>
 *
 * @pure
 */
function array_unique(array $array, int $flags = SORT_STRING): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_intersect(array $array, array ...$arrays): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 *
 * @pure
 */
function array_intersect_key(array $array, array ...$arrays): array
{
}

/**
 * Computes the intersection of arrays using a callback function on the keys for comparison
 * @link https://php.net/manual/en/function.array-intersect-ukey.php
 * @param array $array <p>
 * Initial array for comparison of the arrays.
 * </p>
 * @param array $array2 <p>
 * First array to compare keys against.
 * </p>
 * @param callable $key_compare_func <p>
 * User supplied callback function to do the comparison.
 * </p>
 * @param ...$rest [optional]
 * @return array an array containing all the values of
 * <code>array</code> which have matching keys that are present
 * in all the arguments.
 * @meta
 */
function array_intersect_ukey(array $array, ...$rest): array
{
}

/**
 * Computes the intersection of arrays, compares data by a callback function
 * @link https://php.net/manual/en/function.array-uintersect.php
 * @param array $array <p>
 * The first array.
 * </p>
 * @param array $array2 <p>
 * The second array.
 * </p>
 * @param callable $data_compare_func <p>
 * The callback comparison function.
 * </p>
 * @param array ...$rest
 * <p>
 * The user supplied callback function is used for comparison.
 * It must return an integer less than, equal to, or greater than zero if
 * the first argument is considered to be respectively less than, equal
 * to, or greater than the second.
 * </p>
 * @return array an array containing all the values of <code>array</code>
 * that are present in all the arguments.
 * @meta
 */
function array_uintersect(array $array, ...$rest): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_intersect_assoc(array $array, array ...$arrays): array
{
}

/**
 * Computes the intersection of arrays with additional index check, compares data by a callback function
 * @link https://php.net/manual/en/function.array-uintersect-assoc.php
 * @param array $array <p>
 * The first array.
 * </p>
 * @param array $array2 <p>
 * The second array.
 * </p>
 * @param callable $data_compare_func <p>
 * For comparison is used the user supplied callback function.
 * It must return an integer less than, equal
 * to, or greater than zero if the first argument is considered to
 * be respectively less than, equal to, or greater than the
 * second.
 * </p>
 * @param array ...$rest
 * @return array an array containing all the values of
 * <code>array</code> that are present in all the arguments.
 * @meta
 */
function array_uintersect_assoc(array $array, ...$rest): array
{
}

/**
 * Computes the intersection of arrays with additional index check, compares indexes by a callback function
 * @link https://php.net/manual/en/function.array-intersect-uassoc.php
 * @param array $array <p>
 * Initial array for comparison of the arrays.
 * </p>
 * @param array $array2 <p>
 * First array to compare keys against.
 * </p>
 * @param callable $key_compare_func <p>
 * User supplied callback function to do the comparison.
 * </p>
 * @param array ...$rest
 * @return array the values of <code>array</code> whose values exist in all of the arguments.
 * @meta
 */
function array_intersect_uassoc(array $array, ...$rest): array
{
}

/**
 * Computes the intersection of arrays with additional index check, compares data and indexes by separate callback functions
 * @link https://php.net/manual/en/function.array-uintersect-uassoc.php
 * @param array $array <p>
 * The first array.
 * </p>
 * @param array $array2 <p>
 * The second array.
 * </p>
 * @param callable $data_compare_func <p>
 * For comparison is used the user supplied callback function.
 * It must return an integer less than, equal
 * to, or greater than zero if the first argument is considered to
 * be respectively less than, equal to, or greater than the
 * second.
 * </p>
 * @param callable $key_compare_func <p>
 * Key comparison callback function.
 * </p>
 * @param array ...$rest
 * @return array an array containing all the values and keys of
 * array1 that are present in all the arguments.
 * @meta
 */
#[Pure]
function array_uintersect_uassoc(array $array, ...$rest): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_diff(array $array, array ...$arrays): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_diff_key(array $array, array ...$arrays): array
{
}

/**
 * Computes the difference of arrays using a callback function on the keys for comparison
 * @link https://php.net/manual/en/function.array-diff-ukey.php
 * @param array $array <p>
 * The array to compare from
 * </p>
 * @param array $array2 <p>
 * An array to compare against
 * </p>
 * @param callable $key_compare_func <p>
 * callback function to use.
 * The callback function must return an integer less than, equal
 * to, or greater than zero if the first argument is considered to
 * be respectively less than, equal to, or greater than the second.
 * </p>
 * @param array ...$rest [optional]
 * @return array an array containing all the entries from
 * <code>array</code> that are not present in any of the other arrays.
 * @meta
 */
function array_diff_ukey(array $array, ...$rest): array
{
}

/**
 * Computes the difference of arrays by using a callback function for data comparison
 * @link https://php.net/manual/en/function.array-udiff.php
 * @param array $array <p>
 * The first array.
 * </p>
 * @param array $array2 <p>
 * The second array.
 * </p>
 * @param callable $data_compare_func <p>
 * The callback comparison function.
 * </p>
 * <p>
 * The user supplied callback function is used for comparison.
 * It must return an integer less than, equal to, or greater than zero if
 * the first argument is considered to be respectively less than, equal
 * to, or greater than the second.
 * </p>
 * @param array ...$rest [optional]
 * @return array an array containing all the values of
 * <code>array</code> that are not present in any of the other arguments.
 * @meta
 */
function array_udiff(array $array, ...$rest): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param array ...$arrays
 *
 * @return array<K, V>
 * @pure
 */
function array_diff_assoc(array $array, array ...$arrays): array
{
}

/**
 * Computes the difference of arrays with additional index check, compares data by a callback function
 * @link https://php.net/manual/en/function.array-udiff-assoc.php
 * @param array $array <p>
 * The first array.
 * </p>
 * @param array $array2 <p>
 * The second array.
 * </p>
 * @param callable $data_compare_func <p>
 * The callback comparison function.
 * </p>
 * <p>
 * The user supplied callback function is used for comparison.
 * It must return an integer less than, equal to, or greater than zero if
 * the first argument is considered to be respectively less than, equal
 * to, or greater than the second.
 * </p>
 * @param array ...$rest [optional]
 * @return array returns an array containing all the values from <code>array</code>
 * that are not present in any of the other arguments.
 * Note that the keys are used in the comparison unlike
 * array_diff and array_udiff.
 * The comparison of arrays' data is performed by using an user-supplied
 * callback. In this aspect the behaviour is opposite to the behaviour of
 * array_diff_assoc which uses internal function for
 * comparison.
 * @meta
 */
function array_udiff_assoc(array $array, ...$rest): array
{
}

/**
 * Computes the difference of arrays with additional index check which is performed by a user supplied callback function
 * @link https://php.net/manual/en/function.array-diff-uassoc.php
 * @param array $array <p>
 * The array to compare from
 * </p>
 * @param array $array2 <p>
 * An array to compare against
 * </p>
 * @param callable $key_compare_func <p>
 * callback function to use.
 * The callback function must return an integer less than, equal
 * to, or greater than zero if the first argument is considered to
 * be respectively less than, equal to, or greater than the second.
 * </p>
 * @param array ...$rest [optional]
 * @return array an array containing all the values and keys from
 * <code>array</code> that are not present in any of the other arrays.
 * @meta
 */
function array_diff_uassoc(array $array, ...$rest): array
{
}

/**
 * Computes the difference of arrays with additional index check, compares data and indexes by a callback function
 * @link https://php.net/manual/en/function.array-udiff-uassoc.php
 * @param array $array <p>
 * The first array.
 * </p>
 * @param array $array2 <p>
 * The second array.
 * </p>
 * @param callable $data_compare_func <p>
 * The callback comparison function.
 * </p>
 * <p>
 * The user supplied callback function is used for comparison.
 * It must return an integer less than, equal to, or greater than zero if
 * the first argument is considered to be respectively less than, equal
 * to, or greater than the second.
 * </p>
 * <p>
 * The comparison of arrays' data is performed by using an user-supplied
 * callback : data_compare_func. In this aspect
 * the behaviour is opposite to the behaviour of
 * array_diff_assoc which uses internal function for
 * comparison.
 * </p>
 * @param callable $key_compare_func <p>
 * The comparison of keys (indices) is done also by the callback function
 * key_compare_func. This behaviour is unlike what
 * array_udiff_assoc does, since the latter compares
 * the indices by using an internal function.
 * </p>
 * @param array ...$rest [optional]
 * @return array an array containing all the values and keys from
 * <code>array</code> that are not present in any of the other
 * arguments.
 * @meta
 */
function array_udiff_uassoc(array $array, ...$rest): array
{
}

/**
 * Calculate the sum of values in an array
 * @link https://php.net/manual/en/function.array-sum.php
 * @param array $array <p>
 * The input array.
 * </p>
 * @return int|float the sum of values as an integer or float.
 */
#[Pure]
function array_sum(array $array): int|float
{
}

/**
 * Calculate the product of values in an array
 * @link https://php.net/manual/en/function.array-product.php
 * @param array $array <p>
 * The array.
 * </p>
 * @return int|float the product as an integer or float.
 */
#[Pure]
function array_product(array $array): int|float
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K, V> $array
 * @param null|(callable(V, K): bool)|(callable(V): bool)|(callable(K): bool) $callback
 *
 * @return array<K, V>
 */
function array_filter(array $array, null|callable $callback, int $mode = 0): array
{
}

/**
 * @template K as array-key
 * @template V
 * @template S
 * @template U
 *
 * @param (callable(V, S): U)|null $callback
 * @param array<K, V> $array
 * @param array<array-key, S> ...$arrays
 *
 * @return array<K, U>
 */
function array_map(null|callable $callback, array $array, array ...$arrays): array
{
}

/**
 * Split an array into chunks
 * @link https://php.net/manual/en/function.array-chunk.php
 * @param array $array <p>
 * The array to work on
 * </p>
 * @param int $length <p>
 * The size of each chunk
 * </p>
 * @param bool $preserve_keys [optional] <p>
 * When set to true keys will be preserved.
 * Default is false which will reindex the chunk numerically
 * </p>
 * @return array a multidimensional numerically indexed array, starting with zero,
 * with each dimension containing size elements.
 *
 * @pure
 */
function array_chunk(array $array, int $length, bool $preserve_keys = false): array
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param array<K> $keys
 * @param array<V> $values
 *
 * @return ($keys is non-empty-array ? non-empty-array<K, V> : array<K, V>)
 *
 * @pure
 */
function array_combine(array $keys, array $values): array
{
}

/**
 * @pure
 */
function array_key_exists(string|int|float|bool|null $key, array $array): bool
{
}

/**
 * @template K as array-key
 *
 * @param array<K, mixed> $array
 *
 * @return K|null
 *
 * @pure
 */
function array_key_first(array $array): string|int|null
{
}

/**
 * @template K as array-key
 *
 * @param array<K, mixed> $array
 *
 * @return K|null
 *
 * @pure
 */
function array_key_last(array $array): string|int|null
{
}

/**
 * @pure
 */
function pos(object|array $array): mixed
{
}

/**
 * @return int<0, max>
 *
 * @pure
 */
function sizeof(Countable|array $value, int $mode = COUNT_NORMAL): int
{
}

/**
 * @template K as array-key
 * @template V
 *
 * @param K $key
 * @param array<K, V> $array
 */
function key_exists($key, array $array): bool
{
}

/**
 * @assert truthy $assertion
 */
function assert(mixed $assertion, Throwable|string|null $description = null): bool
{
}

class AssertionError extends Error
{
}

/**
 * @deprecated
 */
function assert_options(int $option, mixed $value): mixed
{
}

/**
 * @param null|'<'|'lt'|'<='|'le'|'>'|'gt'|'>='|'ge'|'=='|'='|'eq'|'!='|'<>'|'ne' $operator
 *
 * @return int<-1, 1>|bool
 *
 * @pure
 */
function version_compare(string $version1, string $version2, null|string $operator): int|bool
{
}

function ftok(string $filename, string $project_id): int
{
}

/**
 * @pure
 */
function str_rot13(string $string): string
{
}

/**
 * @return list<string>
 */
function stream_get_filters(): array
{
}

/**
 * @param resource $stream
 */
function stream_isatty($stream): bool
{
}

/**
 * @param class-string $class
 */
function stream_filter_register(string $filter_name, string $class): bool
{
}

/**
 * @param resource $brigade
 */
function stream_bucket_make_writeable($brigade): StreamBucket|null
{
}

/**
 * @param resource $brigade
 */
function stream_bucket_prepend($brigade, StreamBucket $bucket): void
{
}

/**
 * @param resource $brigade
 */
function stream_bucket_append($brigade, StreamBucket $bucket): void
{
}

/**
 * @param resource $stream
 */
function stream_bucket_new($stream, string $buffer): StreamBucket
{
}

function output_add_rewrite_var(string $name, string $value): bool
{
}

function output_reset_rewrite_vars(): bool
{
}

/**
 * @return non-empty-string
 */
function sys_get_temp_dir(): string
{
}

function realpath_cache_get(): array
{
}

function realpath_cache_size(): int
{
}

function get_mangled_object_vars(object $object): array
{
}

/**
 * @return non-empty-string
 *
 * @pure
 */
function get_debug_type(mixed $value): string
{
}

/**
 * @param resource $resource
 *
 * @pure
 */
function get_resource_id($resource): int
{
}
