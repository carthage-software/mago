<?php

class SimdJsonException extends RuntimeException {}

class SimdJsonValueError extends ValueError {}

/**
 * @throws SimdJsonException
 * @throws SimdJsonValueError
 */
function simdjson_decode(
    string $json,
    bool $associative = false,
    int $depth = 512,
): array|stdClass|string|float|int|bool|null {}

/**
 * @throws SimdJsonValueError
 */
function simdjson_is_valid(string $json, int $depth = 512): bool {}

/**
 * @throws SimdJsonException
 * @throws SimdJsonValueError
 */
function simdjson_key_count(string $json, string $key, int $depth = 512, bool $throw_if_uncountable = false): int {}

/**
 * @throws SimdJsonException
 * @throws SimdJsonValueError
 */
function simdjson_key_exists(string $json, string $key, int $depth = 512): bool {}

/**
 * @throws SimdJsonException
 * @throws SimdJsonValueError
 */
function simdjson_key_value(
    string $json,
    string $key,
    bool $associative = false,
    int $depth = 512,
): array|stdClass|string|float|int|bool|null {}
