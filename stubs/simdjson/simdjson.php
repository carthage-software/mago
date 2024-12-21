<?php

/**
 * Takes a JSON encoded string and converts it into a PHP variable.
 * Similar to json_decode()
 *
 * @param string $json The JSON string being decoded
 * @param bool $associative When true, JSON objects will be returned as associative arrays.
 *                          When false, JSON objects will be returned as objects.
 * @param int $depth the maximum nesting depth of the structure being decoded.
 * @return array|stdClass|string|float|int|bool|null
 * @throws SimdJsonException for invalid JSON
 *                           (or $json over 4GB long, or out of range integer/float)
 * @throws SimdJsonValueError for invalid $depth
 */
function simdjson_decode(string $json, bool $associative = false, int $depth = 512) {}

/**
 * Returns true if json is valid.
 *
 * @param string $json The JSON string being decoded
 * @param int $depth the maximum nesting depth of the structure being decoded.
 * @return bool
 * @throws SimdJsonValueError for invalid $depth
 */
function simdjson_is_valid(string $json, int $depth = 512): bool {}

/**
 * Parses $json and returns the number of keys in $json matching the JSON pointer $key
 *
 * @param string $json The JSON string being decoded
 * @param string $key The JSON pointer being requested
 * @param int $depth The maximum nesting depth of the structure being decoded.
 * @param bool $throw_if_uncountable If true, then throw SimdJsonException instead of
returning 0 for JSON pointers
to values that are neither objects nor arrays.
 * @return int
 * @throws SimdJsonException for invalid JSON or invalid JSON pointer
 *                           (or document over 4GB, or out of range integer/float)
 * @throws SimdJsonValueError for invalid $depth
 * @see https://www.rfc-editor.org/rfc/rfc6901.html
 */
function simdjson_key_count(string $json, string $key, int $depth = 512, bool $throw_if_uncountable = false): int {}

/**
 * Returns true if the JSON pointer $key could be found.
 *
 * @param string $json The JSON string being decoded
 * @param string $key The JSON pointer being requested
 * @param int $depth the maximum nesting depth of the structure being decoded.
 * @return bool (false if key is not found)
 * @throws SimdJsonException for invalid JSON or invalid JSON pointer
 *                           (or document over 4GB, or out of range integer/float)
 * @throws SimdJsonValueError for invalid $depth
 * @see https://www.rfc-editor.org/rfc/rfc6901.html
 */
function simdjson_key_exists(string $json, string $key, int $depth = 512): bool {}

/**
 * Returns the value at the json pointer $key
 *
 * @param string $json The JSON string being decoded
 * @param string $key The JSON pointer being requested
 * @param int $depth the maximum nesting depth of the structure being decoded.
 * @param bool $associative When true, JSON objects will be returned as associative arrays.
 *                          When false, JSON objects will be returned as objects.
 * @return array|stdClass|string|float|int|bool|null the value at $key
 * @throws SimdJsonException for invalid JSON or invalid JSON pointer
 *                           (or document over 4GB, or out of range integer/float)
 * @throws SimdJsonValueError for invalid $depth
 * @see https://www.rfc-editor.org/rfc/rfc6901.html
 */
function simdjson_key_value(string $json, string $key, bool $associative = false, int $depth = 512) {}

/**
 * An error thrown by simdjson when processing json.
 *
 * The error code is available as $e->getCode().
 * This can be compared against the `SIMDJSON_ERR_*` constants.
 *
 * Before simdjson 2.1.0, a regular RuntimeException with an error code of 0 was thrown.
 */
class SimdJsonException extends RuntimeException {}

/**
 * Thrown for error conditions on fields such as $depth that are not expected to be
 * from user-provided JSON, with similar behavior to php 8.0.
 *
 * NOTE: https://www.php.net/valueerror was added in php 8.0.
 * In older php versions, this extends Error instead.
 *
 * When support for php 8.0 is dropped completely,
 * a major release of simdjson will likely switch to a standard ValueError.
 */
class SimdJsonValueError extends ValueError {}
